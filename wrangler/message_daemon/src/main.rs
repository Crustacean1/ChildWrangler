pub mod cancellation;
pub mod levenshtein;
pub mod tests;

use std::env;

use chrono::{Datelike, Months, NaiveDate, NaiveDateTime};
use dto::messages::{Meal, MessageProcessing, RequestError, Student, StudentCancellation, Token};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use sqlx::{Connection, Error, Executor, PgPool, Postgres, postgres::PgListener, types::Json};
use uuid::Uuid;

use crate::{
    cancellation::{construct_response, into_cancellations, into_request, save_attendance},
    levenshtein::levenshtein,
};

async fn fetch_and_process<'a, 'b>(pool: &PgPool) -> Option<()> {
    let mut tr = pool.begin().await.expect("Failed to start transaction");

    let message = sqlx::query!(
        "SELECT id, phone, content, sent FROM messages WHERE NOT processed AND NOT outgoing LIMIT 1 "
    )
    .fetch_optional(&mut *tr)
    .await
    .expect("Failed to retrieve message from db")
    .map(|row| Message {
        id: row.id,
        sender: row.phone,
        content: row.content,
        arrived: row.sent,
    })?;

    log::info!(
        "Received message {}: {} from {}",
        message.id,
        message.content,
        message.sender
    );

    match fetch_message(message.clone(), &mut *tr)
        .await
        .expect("Failed to process message")
    {
        Some(_) => log::info!("Message processed"),
        None => log::info!(
            "Skipped message, no guardian matches phone: {}",
            message.sender
        ),
    }

    log::info!("Done, marking as processed");
    sqlx::query!(
        "UPDATE messages SET processed = true WHERE id = $1",
        message.id
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttendanceCancellation {
    pub students: Vec<StudentCancellation>,
}

pub struct OutMsg {
    pub number: String,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub sender: String,
    pub content: String,
    pub arrived: NaiveDateTime,
}

pub enum Cancellation {
    FullCancellation(AttendanceCancellation),
    PartialCancellation(AttendanceCancellation),
}

pub fn into_token(word: &str, message: &Message, students: &[Student]) -> Token {
    let long_date_regex = Regex::new(r"^((\d{1,2})(-|\.|\/)(\d{1,2})(-|\.|\/)(\d{4}))$").unwrap();
    let middle_date_regex = Regex::new(r"^((\d{1,2})(-|\.|\/)(\d{1,2})(-|\.|\/)(\d{2}))$").unwrap();
    let short_date_regex = Regex::new(r"^((\d{1,2})(-|\.|\/)(\d{1,2}))$").unwrap();

    let meals = students.iter().map(|s| s.meals.iter()).flatten();

    if let Some(date) = long_date_regex.captures(word) {
        let day = date[2].parse();
        let month = date[4].parse();
        let year = date[6].parse();

        if let (Ok(year), Ok(month), Ok(day)) = (year, month, day) {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                Token::Date(date)
            } else {
                Token::Unknown(word.into())
            }
        } else {
            Token::Unknown(word.into())
        }
    } else if let Some(date) = middle_date_regex.captures(word) {
        let day = date[2].parse();
        let month = date[4].parse();
        let year: Result<i32, _> = date[6].parse();

        if let (Ok(year), Ok(month), Ok(day)) = (year, month, day) {
            let current_year = (message.arrived.year() / 10) * 10;
            if let Some(date) = NaiveDate::from_ymd_opt(current_year + year, month, day) {
                Token::Date(date)
            } else {
                Token::Unknown(word.into())
            }
        } else {
            Token::Unknown(word.into())
        }
    } else if let Some(date) = short_date_regex.captures(word) {
        let day = date[2].parse();
        let month = date[4].parse();

        let current_year = message.arrived.year();
        if let (Ok(month), Ok(day)) = (month, day) {
            if let Some(date) = NaiveDate::from_ymd_opt(current_year, month, day) {
                if date < message.arrived.date() {
                    if let Some(next_date) = date.checked_add_months(Months::new(12)) {
                        Token::Date(next_date)
                    } else {
                        Token::Unknown(word.into())
                    }
                } else {
                    Token::Date(date)
                }
            } else {
                Token::Unknown(word.into())
            }
        } else {
            Token::Unknown(word.into())
        }
    } else {
        let meals = meals.map(|meal| (meal.name.clone(), Token::Meal(meal.id)));

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
}

pub async fn message_pipeline<C>(
    students: &Vec<Student>,
    message: &Message,
    conn: &mut C,
) -> Result<(), Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    let id = message.id;

    save_state(MessageProcessing::Context(students.clone()), id, conn).await?;

    save_trigger(message.id, id, conn).await?;

    let tokens = into_tokens(message, students);
    save_state(MessageProcessing::Tokens(tokens.clone()), id, conn).await?;

    let request = into_request(&tokens);

    let message = match request {
        Ok(request) => {
            save_state(MessageProcessing::Cancellation(request.clone()), id, conn).await?;

            let cancellations = into_cancellations(&request, students, message);

            save_state(
                MessageProcessing::StudentCancellation(cancellations.students.clone()),
                id,
                conn,
            )
            .await?;

            let change = save_attendance(cancellations, id, &mut *conn).await?;

            construct_response(&change, &message)
        }
        Err(error) => {
            save_state(MessageProcessing::RequestError(error.clone()), id, conn).await?;
            into_err_msg(&error, &message)
        }
    };

    let out_msg_id = enqueue_message(message, conn).await?;
    //save_trigger(out_msg_id, id, conn).await?;

    Ok(())
}

fn into_err_msg(err: &RequestError, message: &Message) -> OutMsg {
    let content = match err {
        RequestError::InvalidTimeRange => format!("Podano nieprawidłowy zakres dat"),
        RequestError::TooManyDates => format!(
            "Podano zbyt wiele dat - należy podać pojedyńczą date nieobecności, lub okres pomiędzy 2 datami odseparowane spacją"
        ),
        RequestError::NoDateSpecified => format!(
            "Nie podano żadnej daty - należy podać pojedyńczą date nieobecności, lub okres pomiędzy 2 datami odseparowane spacją"
        ),
        RequestError::UnknownTerm => {
            format!("Termin '' nie jest prawidłowym określeniem na posiłek / ucznia")
        }
        RequestError::AmbiguousTerm => {
            format!("Termin '' może odnosić się do więcej niż jednego posiłku / ucznia")
        }
    };
    OutMsg {
        content,
        number: message.sender.clone(),
    }
}

fn into_tokens(message: &Message, students: &[Student]) -> Vec<Token> {
    message
        .content
        .to_lowercase()
        .trim()
        .split_whitespace()
        .map(|word| into_token(word, &message, &students))
        .collect::<Vec<_>>()
}

async fn save_state<C>(state: MessageProcessing, context: Uuid, conn: &mut C) -> Result<(), Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    sqlx::query!(
        "INSERT INTO processing_info (cause_id, value) VALUES ($1,$2)",
        context,
        Json(state) as _
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn save_trigger<C>(msg_id: Uuid, cause_id: Uuid, conn: &mut C) -> Result<(), Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    /*sqlx::query!(
        "INSERT INTO processing_trigger (message_id, processing_id) VALUES ($1,$2)",
        msg_id,
        cause_id
    )
    .execute(conn)
    .await?;*/
    Ok(())
}

pub async fn fetch_message<C>(message: Message, tr: &mut C) -> Result<Option<()>, Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    let Some(guardian_id) = sqlx::query!(
        "SELECT guardians.id FROM guardians WHERE
    guardians.phone = $1  OR format('+48%s', guardians.phone) = $1
",
        message.sender
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
        WHERE guardians.id = $1
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

    message_pipeline(&students, &message, &mut *tr).await?;

    Ok(Some(()))
}

async fn enqueue_message<C>(message: OutMsg, connection: &mut C) -> Result<Uuid, Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    let id = sqlx::query!(
        "INSERT INTO messages (phone, content, outgoing, sent) VALUES ($1,$2,true, NOW()) RETURNING id",
        message.number,
        message.content,
    )
    .fetch_one(&mut *connection)
    .await?
    .id;
    Ok(id)
}
