use std::collections::HashMap;

use leptos::prelude::*;
use uuid::Uuid;

use crate::dtos::{
    catering::GuardianDetailDto,
    messages::{
        ContactDto, GuardianDetails, GuardianDto, GuardianStudent, Message, MessageProcessing,
        MessageType,
    },
};

#[server]
pub async fn get_contacts() -> Result<Vec<ContactDto>, ServerFnError> {
    use sqlx::postgres::PgPool;
    use uuid::Uuid;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let guardians = sqlx::query!("WITH randomz AS (SELECT \"SenderNumber\" AS phone FROM inbox GROUP BY \"SenderNumber\")
        SELECT fullname, COALESCE(guardians.phone,randomz.phone) AS phone, id FROM guardians 
        LEFT JOIN randomz ON randomz.phone = guardians.phone OR randomz.phone = format('+48%s', guardians.phone)")
        .fetch_all(&pool)
    .await?.into_iter()
        .map(|row| {
            ContactDto::GuardianWithPhone(
                GuardianDto{
                    id: row.id,
                    fullname: row.fullname,
                phone: row.phone
                }
            )
        })
    ;

    let unknowns = sqlx::query!("WITH randomz AS (SELECT \"SenderNumber\" AS phone FROM inbox GROUP BY \"SenderNumber\")
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
pub async fn get_guardian_details(id: Uuid) -> Result<GuardianDetails, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let row = sqlx::query!(
        "SELECT guardians.id, guardians.fullname, guardians.phone, ARRAY_AGG((students.id, students.name, students.surname)) AS \"students: Vec<(Uuid,String,String)> \" FROM guardians 
        INNER JOIN student_guardians ON student_guardians.guardian_id = guardians.id
        INNER JOIN students ON students.id = student_guardians.student_id
        WHERE guardians.id=$1
        GROUP BY guardians.id",
        id
    )
    .fetch_one(&pool)
    .await?;
    let mut result = GuardianDetails {
        id: row.id,
        phone: row.phone,
        fullname: row.fullname,
        students: row
            .students
            .map(|students| {
                students
                    .into_iter()
                    .map((|(id, name, surname)| GuardianStudent { id, name, surname }))
                    .collect()
            })
            .unwrap_or(vec![]),
        messages: vec![],
    };

    let inbox = match result.phone.clone() {
        Some(phone) => sqlx::query!(
            "SELECT * FROM inbox WHERE \"SenderNumber\" = $1 OR \"SenderNumber\" = format('+48%s',$1)",
            &format!("%{}", &phone)
        )
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| Message {
            id: row.ID,
            sent: row.ReceivingDateTime,
            content: row.TextDecoded,
            msg_type: MessageType::Received(row.Processed),
        })
        .collect(),
        None => vec![],
    };

    let outbox = match result.phone.clone() {
        Some(phone) => sqlx::query!(
            "SELECT * FROM sentitems WHERE \"DestinationNumber\" = $1 OR \"DestinationNumber\" = format('+48%s',$1)",
            &format!("%{}", &phone)
        )
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| Message {
            id: row.ID,
            sent: row.SendingDateTime,
            content: row.TextDecoded,
            msg_type: MessageType::Sent,
        })
        .collect(),
        None => vec![],
    };

    result.messages = inbox.into_iter().chain(outbox.into_iter()).collect();
    result.messages.sort_by(|a, b| a.sent.cmp(&b.sent));

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
pub async fn send_message(phone: String, content: String) -> Result<i32, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let id = sqlx::query!("INSERT INTO outbox (\"TextDecoded\", \"DestinationNumber\", \"CreatorID\") VALUES ($1,$2, 2137) RETURNING \"ID\"", content, phone).fetch_one(&pool).await?.ID;

    Ok(id)
}

#[server]
pub async fn get_messages(phone: String) -> Result<Vec<Message>, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let inbox = sqlx::query!(
        "SELECT * FROM inbox WHERE \"SenderNumber\" LIKE $1",
        &format!("%{}", &phone)
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| Message {
        id: row.ID,
        sent: row.ReceivingDateTime,
        content: row.TextDecoded,
        msg_type: MessageType::Received(row.Processed),
    });

    let sent = sqlx::query!(
        "SELECT * FROM sentitems WHERE \"DestinationNumber\" LIKE $1",
        &format!("%{}", &phone)
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| Message {
        id: row.ID,
        sent: row.SendingDateTime,
        content: row.TextDecoded,
        msg_type: MessageType::Sent,
    });

    let pending = sqlx::query!(
        "SELECT * FROM outbox WHERE \"DestinationNumber\" LIKE $1",
        &format!("%{}", &phone)
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| Message {
        id: row.ID,
        sent: row.SendingDateTime,
        content: row.TextDecoded,
        msg_type: MessageType::Pending,
    });

    Ok(inbox.chain(sent).chain(pending).collect())
}

#[server]
pub async fn requeue_message(msg_id: i32) -> Result<(), ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    sqlx::query!(
        "UPDATE inbox SET \"Processed\" = false WHERE \"ID\" = $1",
        msg_id
    )
    .execute(&mut *tr)
    .await?;

    tr.commit().await?;
    Ok(())
}

#[server]
pub async fn get_message_processing_info(
    msg: i32,
) -> Result<HashMap<Uuid, Vec<MessageProcessing>>, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let processing_info = sqlx::query!(
        "SELECT processing_info.* FROM processing_trigger 
    INNER JOIN processing_info ON processing_info.cause_id = processing_trigger.processing_id
    WHERE message_id = $1
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
