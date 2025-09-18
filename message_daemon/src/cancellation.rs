use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
};

use child_wrangler::dtos::messages::{CancellationRequest, CancellationResult, RequestError};
use chrono::{Days, NaiveTime};
use itertools::Itertools;
use sqlx::{Connection, Error, Executor, Postgres};
use uuid::Uuid;

use crate::{AttendanceCancellation, Message, OutMsg, Student, StudentCancellation, Token};

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
        INNER JOIN generate_series(LEAST(GREATEST($3::date,caterings.since),caterings.until),LEAST(GREATEST($4::date,caterings.since),caterings.until), '1 DAY') AS days(day) ON (caterings.dow >> EXTRACT(DOW FROM day)::integer)&1 = 1
", student.id, &student.meals, student.since, student.until, cause_id).execute(&mut *connection).await?;
    }

    let effective_attendance = sqlx::query!(
        "WITH affected_attendance AS (SELECT DISTINCT ON (ea.day,ea.meal_id,ea.target) ea.day, ea.meal_id, ea.target,group_relations.level FROM attendance AS src
        INNER JOIN group_relations ON group_relations.child = src.target
        INNER JOIN effective_attendance AS ea ON ea.day = src.day AND ea.meal_id = src.meal_id AND ea.target= group_relations.parent
        WHERE src.cause_id  = $1 AND ea.value = false
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

pub fn construct_response(changes: &[CancellationResult], message: &Message) -> OutMsg {
    if !changes.iter().any(|s| s.meals.iter().any(|(_, m)| *m != 0)) {
        OutMsg {
            content: format!("Nie odwołano żadnej obecności"),
            number: message.sender.clone(),
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
        OutMsg {
            content: format!("Odwołano: \n{}", info),
            number: message.sender.clone(),
        }
    }
}

pub fn into_request(tokens: &[Token]) -> Result<CancellationRequest, RequestError> {
    if tokens.iter().any(|token| match token {
        Token::Unknown(_) => true,
        _ => false,
    }) {
        Err(RequestError::UnknownTerm)
    } else if tokens.iter().any(|token| match token {
        Token::Ambiguous(_) => true,
        _ => false,
    }) {
        Err(RequestError::AmbiguousTerm)
    } else {
        let dates = tokens
            .iter()
            .filter_map(|token| match token {
                Token::Date(date) => Some(date),
                _ => None,
            })
            .collect::<Vec<_>>();

        let students = tokens
            .iter()
            .filter_map(|token| match token {
                Token::Student(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();

        let meals = tokens
            .iter()
            .filter_map(|token| match token {
                Token::Meal(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();

        let (since, until) = match dates.len() {
            0 => Err(RequestError::NoDateSpecified),
            1 => Ok((*dates[0], *dates[0])),
            2 => {
                let (since, until) = (*dates[0], *dates[1]);
                if until < since {
                    Err(RequestError::InvalidTimeRange)
                } else {
                    Ok((since, until))
                }
            }
            _ => Err(RequestError::TooManyDates),
        }?;

        Ok(CancellationRequest {
            since,
            until,
            students,
            meals,
        })
    }
}

pub fn into_cancellations(
    request: &CancellationRequest,
    students: &[Student],
    message: &Message,
) -> AttendanceCancellation {
    let request_meals: HashSet<_> = request.meals.iter().collect();

    AttendanceCancellation {
        students: students
            .into_iter()
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

                let since = max(student.starts, min(student.ends, request.since));
                let until = max(student.starts, min(student.ends, request.until));

                let min_allowed = (message.arrived
                    - student
                        .grace_period
                        .signed_duration_since(NaiveTime::default()))
                .date()
                .checked_add_days(Days::new(1))?;

                let since = max(min_allowed, since);

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
            .collect(),
    }
}
