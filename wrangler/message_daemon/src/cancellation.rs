use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
};

use chrono::{Days, NaiveDateTime, NaiveTime};
use dto::messages::{
    AttendanceCancellation, CancellationRequest, CancellationResult, MessageData, MessageMetadata,
    MessageProcessing, ReceivedMessage, RequestError,
};
use itertools::Itertools;
use sqlx::{Connection, Error, Executor, Postgres};
use uuid::Uuid;

use crate::{Student, StudentCancellation, Token};

pub async fn save_attendance<C>(
    request: AttendanceCancellation,
    cause_id: Uuid,
    connection: &mut C,
) -> Result<Vec<CancellationResult>, Error>
where
    C: Connection<Database = Postgres>,
    for<'a> &'a mut C: Executor<'a, Database = Postgres>,
{
    for student in request.students {
        sqlx::query!("INSERT INTO attendance (cause_id, target, day, meal_id, value) SELECT $5, $1, day, meals.id, false FROM UNNEST($2::uuid[]) AS meals(id)
        INNER JOIN group_relations ON group_relations.child = $1
        INNER JOIN caterings ON caterings.group_id = group_relations.parent
        INNER JOIN catering_meals ON catering_meals.meal_id = meals.id AND catering_meals.catering_id = caterings.id
        INNER JOIN generate_series(LEAST(GREATEST($3::date,caterings.since),caterings.until),LEAST(GREATEST($4::date,caterings.since),caterings.until), '1 DAY') AS days(day) ON (caterings.dow >> (EXTRACT(DOW FROM day)::integer + 6) % 7)&1 = 1",
            student.id, &student.meals, student.since, student.until, cause_id)
            .execute(&mut *connection)
            .await?;
    }

    let effective_attendance = sqlx::query!(
        "WITH exclusive_attendance AS (SELECT DISTINCT ON (day, meal_id, target) day, meal_id, target, value FROM attendance WHERE cause_id != $1 ORDER BY day, meal_id, target, originated DESC),
        affected_attendance AS (SELECT DISTINCT ON (ea.day,ea.meal_id,ea.target) ea.day, ea.meal_id, ea.target,group_relations.level FROM attendance AS src
        INNER JOIN group_relations ON group_relations.child = src.target
        INNER JOIN exclusive_attendance AS ea ON ea.day = src.day AND ea.meal_id = src.meal_id AND ea.target= group_relations.parent
        WHERE src.cause_id = $1 AND ea.value = true
        ORDER BY ea.day, ea.meal_id, ea.target, group_relations.level)
        SELECT students.name AS student_name, meals.name AS meal_name, COUNT(*) AS cancelled FROM affected_attendance 
        INNER JOIN students ON students.id = affected_attendance.target
        INNER JOIN meals ON meals.id = affected_attendance.meal_id
        WHERE level = 0
        GROUP BY students.id, meals.id",
        cause_id
    ).fetch_all(&mut*connection).await?;

    let mut hashmap = HashMap::new();
    for attendance in effective_attendance {
        hashmap
            .entry(attendance.student_name)
            .or_insert(HashMap::new())
            .insert(attendance.meal_name, attendance.cancelled.unwrap_or(0_i64));
    }

    Ok(hashmap
        .into_iter()
        .map(|(k, v)| CancellationResult { name: k, meals: v })
        .collect())
}

pub fn construct_response(
    changes: &[CancellationResult],
    message: &ReceivedMessage,
) -> MessageData {
    if !changes.iter().any(|s| s.meals.iter().any(|(_, m)| *m != 0)) {
        MessageData {
            content: format!("Nie odwołano żadnej obecności"),
            phone: message.data.phone.clone(),
        }
    } else {
        let info = changes
            .iter()
            .map(|student| {
                format!(
                    "{}: {}",
                    student.name,
                    student
                        .meals
                        .iter()
                        .filter_map(|(name, count)| match count {
                            0 => None,
                            cnt => {
                                Some(format!("{} {}", name, cnt))
                            }
                        })
                        .join(", ")
                )
            })
            .join("\n");
        MessageData {
            content: format!("Odwołano: \n{}", info),
            phone: message.data.phone.clone(),
        }
    }
}

pub fn into_request(tokens: &[Token]) -> MessageProcessing {
    let mut dates = vec![];
    let mut student_ids = vec![];
    let mut meals = vec![];

    for token in tokens {
        match token {
            Token::Student(uuid) => student_ids.push(*uuid),
            Token::Date(naive_date) => dates.push(*naive_date),
            Token::Meal(uuid) => meals.push(*uuid),
            Token::Unknown(unknown) => {
                return MessageProcessing::RequestError(RequestError::UnknownTerm(unknown.clone()));
            }
            Token::Ambiguous(ambiguous) => {
                return MessageProcessing::RequestError(RequestError::AmbiguousTerm(
                    ambiguous.clone(),
                ));
            }
        }
    }

    let range = match dates.len() {
        0 => Err(RequestError::NoDateSpecified),
        1 => Ok((dates[0], dates[0])),
        2 => {
            let (since, until) = (dates[0], dates[1]);
            if until < since {
                Err(RequestError::InvalidTimeRange)
            } else {
                Ok((since, until))
            }
        }
        _ => Err(RequestError::TooManyDates),
    };

    match range {
        Ok((since, until)) => MessageProcessing::Cancellation(CancellationRequest {
            since,
            until,
            students: student_ids,
            meals,
        }),
        Err(error) => MessageProcessing::RequestError(error),
    }
}

pub fn into_cancellations(
    request: &CancellationRequest,
    students: &[Student],
    received: NaiveDateTime,
) -> AttendanceCancellation {
    let request_meals: HashSet<_> = request.meals.iter().collect();

    let default_student = request.students.is_empty() && students.len() == 1;

    let students = students
        .into_iter()
        .filter(|s| default_student || request.students.iter().any(|s2| *s2 == s.id))
        .filter_map(|student| {
            let student_meals: Vec<_> = student.meals.iter().map(|m| m.id).collect();
            let meals = if request_meals.iter().any(|_| true) {
                student_meals
                    .into_iter()
                    .filter(|m| request_meals.contains(&m))
                    .collect()
            } else {
                student_meals
            };

            let min_grace_period = (received
                - student
                    .grace_period
                    .signed_duration_since(NaiveTime::default()))
            .date()
            .checked_add_days(Days::new(1))?;
            let min_allowed = max(min_grace_period, student.starts);

            let since = max(min_allowed, min(student.ends, request.since));
            let until = max(min_allowed, min(student.ends, request.until));

            if since > until {
                None
            } else {
                Some(StudentCancellation {
                    id: student.id,
                    meals,
                    since,
                    until,
                })
            }
        })
        .collect();

    AttendanceCancellation { students }
}
