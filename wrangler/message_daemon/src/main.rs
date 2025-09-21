pub mod cancellation;
pub mod levenshtein;
pub mod tests;

use std::{env, time::Duration};

use child_wrangler::dtos::messages::{
    Meal, MessageProcessing, RequestError, Student, StudentCancellation, Token,
};
use chrono::{Datelike, Months, NaiveDate, NaiveDateTime};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Error, Executor, PgPool, Postgres, types::Json};
use tokio::time::sleep;
use uuid::Uuid;

use crate::{
    cancellation::{construct_response, into_cancellations, into_request, save_attendance},
    levenshtein::levenshtein,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL was not set");
    let polling_interval = env::var("POLLING_INTERVAL_MS")
        .ok()
        .and_then(|interval| interval.parse().ok())
        .unwrap_or(1000);

    let pool = PgPool::connect(&db_url)
        .await
        .expect("Failed to connect to postgres database");

    loop {
        match fetch_message(&pool).await {
            Ok(message) => {
                if let Some(_) = message {
                    println!("Message processed")
                } else {
                    sleep(Duration::from_millis(polling_interval)).await;
                }
            }
            Err(e) => {
                log::error!("Failed to process message: {}", e);
                sleep(Duration::from_millis(polling_interval)).await;
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

pub struct Message {
    pub id: i32,
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
        println!("Matching the regex");
        let day = date[2].parse();
        let month = date[4].parse();

        let current_year = message.arrived.year();
        println!("iiiiiFick3: {:?} {:?}", &date[2], &date[4]);
        if let (Ok(month), Ok(day)) = (month, day) {
            if let Some(date) = NaiveDate::from_ymd_opt(current_year, month, day) {
                if date < message.arrived.date() {
                    if let Some(next_date) = date.checked_add_months(Months::new(12)) {
                        Token::Date(next_date)
                    } else {
                        println!("Fick1");
                        Token::Unknown(word.into())
                    }
                } else {
                    Token::Date(date)
                }
            } else {
                println!("Fick2");

                Token::Unknown(word.into())
            }
        } else {
            println!("Fick3: ");
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
    let id = Uuid::new_v4();

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
            "Podano zbyt wiele dat - należy podać pojedyńczą date nieobecności, lub okres pomiędzy 2 datami"
        ),
        RequestError::NoDateSpecified => format!(
            "Nie podano żadnej daty - należy podać pojedyńczą date nieobecności, lub okres pomiędzy 2 datami"
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

async fn save_trigger<C>(msg_id: i32, cause_id: Uuid, conn: &mut C) -> Result<(), Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    sqlx::query!(
        "INSERT INTO processing_trigger (message_id, processing_id) VALUES ($1,$2)",
        msg_id,
        cause_id
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn fetch_message(pool: &PgPool) -> Result<Option<()>, Error> {
    let mut tr = pool.begin().await?;
    let Some((message,guardian_id)) = sqlx::query!(
        "SELECT inbox.*, guardians.id AS guardian_id FROM inbox 
            INNER JOIN guardians ON guardians.phone = inbox.\"SenderNumber\"  OR inbox.\"SenderNumber\" = format('+48%s', guardians.phone)
            WHERE \"Processed\" = false FOR UPDATE SKIP LOCKED LIMIT 1"
    )
    .fetch_optional(&mut *tr)
    .await?
    .map(|row| (Message {
        id: row.ID,
        content: row.TextDecoded,
        sender: row.SenderNumber,
        arrived: row.ReceivingDateTime,
    },row.guardian_id)) else {
    println!("No message detected :(");
        return Ok(None);
    };

    println!("Message detected!");

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

    sqlx::query!(
        "UPDATE inbox SET \"Processed\" = true WHERE \"ID\" = $1",
        message.id
    )
    .execute(&mut *tr)
    .await?;

    tr.commit().await?;

    Ok(Some(()))
}

async fn enqueue_message<C>(message: OutMsg, connection: &mut C) -> Result<i32, Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    let id = sqlx::query!("INSERT INTO outbox (\"TextDecoded\", \"DestinationNumber\", \"CreatorID\") VALUES ($1,$2, 2137) RETURNING \"ID\"", message.content, message.number).fetch_one(&mut*connection).await?.ID;
    Ok(id)
}
