use std::collections::HashMap;

use chrono::TimeDelta;
use dto::{
    guardian::{GuardianDetailDto, GuardianDto},
    messages::{parse_message, ContactDto, DbMessage, Message, MessageProcessing, PhoneStatusDto},
    student::StudentDto,
};
use leptos::prelude::*;
use uuid::Uuid;

use crate::components::snackbar::MsgType;

#[server]
pub async fn get_contacts() -> Result<Vec<ContactDto>, ServerFnError> {
    use sqlx::postgres::PgPool;
    use uuid::Uuid;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let guardians = sqlx::query!("WITH randomz AS (SELECT DISTINCT phone FROM messages)
        SELECT fullname, COALESCE(guardians.phone,randomz.phone) AS phone, id FROM guardians 
        LEFT JOIN randomz ON randomz.phone = guardians.phone OR randomz.phone = format('+48%s', guardians.phone)")
        .fetch_all(&pool)
    .await?.into_iter()
        .map(|row| {
            ContactDto::GuardianWithPhone(
                GuardianDto{
                    id: row.id,
                    fullname: row.fullname,
                phone: row.phone,
                }
            )
        })
    ;

    let unknowns = sqlx::query!("WITH randomz AS (SELECT DISTINCT phone FROM messages)
        SELECT fullname, randomz.phone AS phone, id FROM randomz 
        LEFT JOIN guardians ON randomz.phone = guardians.phone OR randomz.phone = format('+48%s', guardians.phone)
        WHERE guardians.phone IS NULL")
        .fetch_all(&pool)
    .await?.into_iter()
        .map(|row| {
            ContactDto::Unknown(
                row.phone
            )
        })
    ;

    return Ok(guardians.chain(unknowns).collect());
}

#[server]
pub async fn get_guardian_details(id: Uuid) -> Result<GuardianDetailDto, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let guardian = sqlx::query!(
        "SELECT guardians.id, guardians.fullname, guardians.phone FROM guardians
        WHERE guardians.id=$1",
        id
    )
    .fetch_one(&pool)
    .await?;

    let students = sqlx::query!(
        "SELECT students.id, students.name, students.surname, group_relations.parent AS group_id FROM guardians 
        INNER JOIN student_guardians ON student_guardians.guardian_id = guardians.id
        INNER JOIN students ON students.id = student_guardians.student_id
        INNER JOIN group_relations ON group_relations.child = students.id AND group_relations.level = 1
        WHERE guardians.id=$1",
        id
    )
    .fetch_all(&pool)
    .await?;

    let mut result = GuardianDetailDto {
        id: guardian.id,
        phone: guardian.phone,
        fullname: guardian.fullname,
        students: students
            .into_iter()
            .map(
                (|row| StudentDto {
                    id: row.id,
                    name: row.name,
                    surname: row.surname,
                    group_id: row.group_id,
                }),
            )
            .collect(),
    };

    Ok(result)
}

#[server]
pub async fn update_guardian(guardian: GuardianDetailDto) -> Result<(), ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    let affected = sqlx::query!(
        "UPDATE guardians SET phone=$2 , fullname=$3 WHERE id = $1",
        guardian.id,
        guardian.phone,
        guardian.fullname,
    )
    .execute(&mut *tr)
    .await?;
    if affected.rows_affected() != 1 {
        return Err(ServerFnError::new("Failed to update guardian"));
    }
    tr.commit().await?;
    Ok(())
}

#[server]
pub async fn send_message(phone: String, content: String) -> Result<Uuid, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let id = sqlx::query!(
        "INSERT INTO messages (phone, content, outgoing) VALUES ($1,$2,true) RETURNING id",
        phone,
        content
    )
    .fetch_one(&pool)
    .await?
    .id;

    Ok(id)
}

#[server]
pub async fn get_messages(phone: String) -> Result<Vec<Message>, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let messages = sqlx::query_as!(
        DbMessage,
        "SELECT * FROM messages WHERE phone = $1 OR phone = format('+48%s', $1) ORDER BY inserted",
        phone
    )
    .fetch_all(&pool)
    .await?;
    let messages = messages.into_iter().map(parse_message);

    Ok(messages.collect())
}

#[server]
pub async fn requeue_message(msg_id: Uuid) -> Result<(), ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    sqlx::query!(
        "UPDATE messages SET processed = false WHERE id = $1",
        msg_id
    )
    .execute(&mut *tr)
    .await?;

    tr.commit().await?;
    Ok(())
}

#[server]
pub async fn get_message_processing_info(
    msg: Uuid,
) -> Result<HashMap<Uuid, Vec<MessageProcessing>>, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let processing_info = sqlx::query!(
        "SELECT processing_step.* FROM processing_step 
    WHERE cause_id = $1
    ORDER BY id",
        msg
    )
    .fetch_all(&pool)
    .await?;

    let mut result = HashMap::new();

    for info in processing_info {
        if let Ok(process) = serde_json::from_value(info.value) {
            result.entry(info.cause_id).or_insert(vec![]).push(process);
        }
    }

    Ok(result)
}

#[server]
pub async fn get_phone_status() -> Result<Option<PhoneStatusDto>, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    Ok(None)
}

#[server]
pub async fn get_latest_messages(time_span: TimeDelta) -> Result<Vec<Message>, ServerFnError> {
    use sqlx::postgres::types::PgInterval;
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let interval: PgInterval = time_span.try_into().unwrap();

    let messages = sqlx::query_as!(
        DbMessage,
        "SELECT * FROM messages WHERE NOW() - inserted < $1 ORDER BY inserted",
        interval
    )
    .fetch_all(&pool)
    .await?;

    let messages = messages.into_iter().map(parse_message);

    Ok(messages.collect())
}

#[server]
pub async fn simulate_message(from: String, content: String) -> Result<(), ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    sqlx::query!(
        "INSERT INTO messages (phone, content, outgoing, sent) VALUES ($1,$2,false,NOW())",
        from,
        content
    )
    .execute(&pool)
    .await?;

    Ok(())
}
