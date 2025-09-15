use leptos::prelude::*;
use uuid::Uuid;

use crate::dtos::{
    catering::GuardianDetailDto,
    messages::{ContactDto, GuardianDetails, GuardianDto, GuardianStudent, Message, MessageType},
};

#[server]
pub async fn get_contacts() -> Result<Vec<ContactDto>, ServerFnError> {
    use sqlx::postgres::PgPool;
    use uuid::Uuid;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let results = sqlx::query!("WITH randomz AS (SELECT \"SenderNumber\" AS phone FROM inbox GROUP BY \"SenderNumber\")
        SELECT fullname, COALESCE(guardians.phone,randomz.phone) AS phone, id FROM guardians FULL OUTER JOIN randomz ON randomz.phone = guardians.phone")
        .fetch_all(&pool)
    .await?
        .into_iter()
    .filter_map(|row| {
        match (row.id, row.phone, row.fullname){
                (Some(id),phone, Some(fullname)) => Some(ContactDto::GuardianWithPhone(GuardianDto{id,fullname,phone})),
                (None,Some(phone),None) => Some(ContactDto::Unknown(phone)),
                _ => None
    }})
            .collect();
    return Ok(results);
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
        })
        .collect(),
        None => vec![],
    };

    let outbox = match result.phone.clone() {
        Some(phone) => sqlx::query!(
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
