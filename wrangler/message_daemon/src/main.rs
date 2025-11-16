pub mod cancellation;
pub mod levenshtein;
pub mod tests;

use std::env;

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use dto::messages::{
    DbMessage, Meal, Message, MessageData, MessageProcessing, ReceivedMessage, RequestError,
    Student, StudentCancellation, Token, parse_message,
};
use itertools::Itertools;
use regex::{Match, Regex};
use simple_logger::SimpleLogger;
use sqlx::{Connection, Error, Executor, PgPool, Postgres, postgres::PgListener, types::Json};
use uuid::Uuid;

use crate::{
    cancellation::{construct_response, into_cancellations, into_request, save_attendance},
    levenshtein::levenshtein,
};

async fn fetch_and_process<'a, 'b>(pool: &PgPool) -> Option<()> {
    let mut tr = pool.begin().await.expect("Failed to start transaction");

    let message = sqlx::query_as!(
        DbMessage,
        r#"SELECT messages.* FROM messages 
        WHERE NOT outgoing AND NOT processed 
        LIMIT 1 FOR UPDATE SKIP LOCKED"#
    )
    .fetch_optional(&mut *tr)
    .await
    .expect("Failed to retrieve message from db")?;

    let message = parse_message(message);

    log::info!("Received message notification: {:?}", message);

    match &message {
        Message::Received(message) => {
            match fetch_message(message.clone(), &mut *tr)
                .await
                .expect("Failed to process message")
            {
                Some(_) => log::info!("Message processed"),
                None => log::info!(
                    "Skipped message, no guardian matches phone: {}",
                    message.data.phone
                ),
            }
        }
        _ => {
            log::warn!(
                "Message is not recognized as incoming, skipping processing (this shouldn't have happened, msg_id: {})",
                message.metadata().id
            )
        }
    }

    log::info!("Done, marking as processed");
    sqlx::query!(
        "UPDATE messages SET processed = true WHERE id = $1",
        message.metadata().id
    )
    .execute(&mut *tr)
    .await
    .expect("Failed to mark message as processed, rollback");

    tr.commit().await.expect("Failed to commit transaction");
    Some(())
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL was not set");

    log::info!("Using db connection: {}", db_url);

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to postgres database");

    let mut listener = PgListener::connect_with(&pool)
        .await
        .expect("Failed to connect to postgres events");

    listener
        .listen("received")
        .await
        .expect("Failed to start listening on 'received' channel");

    while let Some(_) = fetch_and_process(&pool).await {
        log::info!("Processed stale message");
    }

    log::info!("Caught up to latest messages, starting listening");
    loop {
        match listener.recv().await {
            Ok(event) => {
                log::info!("Received notification for incoming message: {:?}", event);
                while let Some(_) = fetch_and_process(&pool).await {
                    log::info!("Processed message");
                }
                log::info!("Done processing, waiting for next notification");
            }
            Err(e) => {
                log::warn!("Failure while listening for events: {}", e);
            }
        }
    }
}

fn into_date(
    current: NaiveDateTime,
    day: Option<Match>,
    month: Option<Match>,
    year: Option<Match>,
) -> Option<NaiveDate> {
    let day: u32 = day?.as_str().parse().ok()?;
    let month: u32 = month?.as_str().parse().ok()?;
    if let Some(year) = year {
        let year: i32 = year.as_str().parse().ok()?;
        NaiveDate::from_ymd_opt(year, month, day)
    } else {
        let current_date = NaiveDate::from_ymd_opt(current.year(), month, day)?;
        let next_date = NaiveDate::from_ymd_opt(current.year() + 1, month, day)?;
        if (current_date - current.date()) < (next_date - current.date()) {
            Some(current_date)
        } else {
            Some(next_date)
        }
    }
}

pub fn into_token(word: &str, message: &ReceivedMessage, students: &[Student]) -> Token {
    let long_date_regex = Regex::new(r"^((\d{1,2})(-|\.|\/)(\d{1,2})(-|\.|\/)(\d{4}))$").unwrap();
    let middle_date_regex = Regex::new(r"^((\d{1,2})(-|\.|\/)(\d{1,2})(-|\.|\/)(\d{2}))$").unwrap();
    let short_date_regex = Regex::new(r"^((\d{1,2})(-|\.|\/)(\d{1,2}))$").unwrap();

    let regexes = [long_date_regex, middle_date_regex, short_date_regex];

    for date_regex in regexes {
        if let Some(date) = date_regex.captures(word) {
            if let Some(date) = into_date(message.received, date.get(2), date.get(4), date.get(6)) {
                return Token::Date(date);
            } else {
                return Token::Unknown(word.into());
            }
        }
    }

    let meals = students
        .iter()
        .map(|s| s.meals.iter())
        .flatten()
        .map(|meal| (meal.name.clone(), Token::Meal(meal.id)));

    let students = students
        .iter()
        .map(|student| (student.name.clone(), Token::Student(student.id)));

    let target = meals
        .chain(students)
        .filter(|(name, _)| levenshtein(&name, word) <= 3)
        .min_set_by_key(|(name, _)| levenshtein(&name, word));

    match target.len() {
        0 => Token::Unknown(word.into()),
        1 => target[0].1.clone(),
        _ => Token::Ambiguous(word.into()),
    }
}

pub async fn pipeline<C>(
    context: (ReceivedMessage, Vec<Student>, MessageProcessing),
    conn: &mut C,
) -> Result<Option<(ReceivedMessage, Vec<Student>, MessageProcessing)>, Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    let (message, students, processing) = context;

    let new = match processing {
        MessageProcessing::Init => {
            Some(MessageProcessing::Tokens(into_tokens(&message, &students)))
        }
        MessageProcessing::Tokens(tokens) => Some(into_request(&tokens)),
        MessageProcessing::Cancellation(cancellation_request) => {
            Some(MessageProcessing::StudentCancellation(into_cancellations(
                &cancellation_request,
                &students,
                message.received,
            )))
        }
        MessageProcessing::StudentCancellation(student_cancellations) => {
            Some(MessageProcessing::CancellationResult(
                save_attendance(student_cancellations, message.metadata.id, conn).await?,
            ))
        }
        MessageProcessing::CancellationResult(results) => {
            let response = construct_response(&results, &message);
            enqueue_message(response, message.metadata.id, conn).await?;
            None
        }
        MessageProcessing::RequestError(request_error) => {
            let response = construct_err_response(&request_error, &message);
            enqueue_message(response, message.metadata.id, conn).await?;
            None
        }
    };

    if let Some(new) = new {
        save_state(&new, message.metadata.id, conn).await?;
        return Ok(Some((message, students, new)));
    } else {
        Ok(None)
    }
}

fn construct_err_response(err: &RequestError, message: &ReceivedMessage) -> MessageData {
    let content = match err {
        RequestError::InvalidTimeRange => format!("Podano nieprawidłowy zakres dat"),
        RequestError::TooManyDates => format!(
            "Podano zbyt wiele dat - należy podać pojedyńczą date nieobecności, lub okres pomiędzy 2 datami odseparowane spacją"
        ),
        RequestError::NoStudentSpecified => format!("Nie podano ucznia"),
        RequestError::NoDateSpecified => format!(
            "Nie podano żadnej daty - należy podać pojedyńczą date nieobecności, lub okres pomiędzy 2 datami odseparowane spacją"
        ),
        RequestError::UnknownTerm(term) => {
            format!(
                "Termin '{}' nie jest prawidłowym określeniem na posiłek / ucznia",
                term
            )
        }
        RequestError::AmbiguousTerm(term) => {
            format!(
                "Termin '{}' może odnosić się do więcej niż jednego posiłku / ucznia",
                term
            )
        }
    };
    MessageData {
        phone: message.data.phone.clone(),
        content,
    }
}

fn into_tokens(message: &ReceivedMessage, students: &[Student]) -> Vec<Token> {
    message
        .data
        .content
        .to_lowercase()
        .trim()
        .split_whitespace()
        .map(|word| into_token(word, &message, &students))
        .collect::<Vec<_>>()
}

async fn save_state<C>(state: &MessageProcessing, context: Uuid, conn: &mut C) -> Result<(), Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    sqlx::query!(
        "INSERT INTO processing_step (cause_id, value) VALUES ($1,$2)",
        context,
        Json(state) as _
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn fetch_message<C>(message: ReceivedMessage, tr: &mut C) -> Result<Option<()>, Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    let Some(guardian_id) = sqlx::query!(
        "SELECT guardians.id FROM guardians WHERE
    guardians.phone = $1  OR format('+48%s', guardians.phone) = $1
",
        message.data.phone
    )
    .fetch_optional(&mut *tr)
    .await?
    .map(|row| row.id) else {
        return Ok(None);
    };

    let students = sqlx::query!(
        "SELECT students.id, students.name, students.surname, caterings.grace_period, ARRAY_AGG((meals.id,meals.name)) AS \"meals: Vec<(Uuid,String)> \", caterings.since, caterings.until FROM students 
        INNER JOIN student_guardians ON student_guardians.student_id = students.id
        INNER JOIN guardians ON student_guardians.guardian_id = guardians.id
        INNER JOIN group_relations ON group_relations.child = students.id
        INNER JOIN caterings ON caterings.group_id = group_relations.parent
        INNER JOIN catering_meals ON catering_meals.catering_id = caterings.id
        INNER JOIN meals ON meals.id = catering_meals.meal_id
        WHERE guardians.id = $1 AND NOT students.removed
        GROUP BY students.id,caterings.id",
        guardian_id
    )
    .fetch_all(&mut *tr)
    .await?
    .into_iter()
    .map(|row| Student {
        id: row.id,
        name: row.name,
        surname: row.surname,
        meals: row.meals.map(|meals| meals.into_iter().map(|(id,name)| Meal{name,id}).collect()).unwrap_or(vec![]),
        grace_period: row.grace_period,
            starts: row.since,
            ends: row.until
    })
    .collect::<Vec<_>>();

    let mut context = (message, students, MessageProcessing::Init);

    loop {
        if let Some(new_context) = pipeline(context, tr).await? {
            context = new_context;
        } else {
            break;
        }
    }

    Ok(Some(()))
}

async fn enqueue_message<C>(
    message: MessageData,
    cause_id: Uuid,
    connection: &mut C,
) -> Result<Uuid, Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    let id = sqlx::query!(
        "INSERT INTO messages (phone, content, outgoing, cause_id) VALUES ($1, $2, true, $3) RETURNING id",
        message.phone,
        message.content,
        cause_id
    )
    .fetch_one(&mut *connection)
    .await?
    .id;
    Ok(id)
}
