use dto::catering::{CateringDto, CreateCateringDto};
use leptos::logging::log;
use leptos::prelude::*;
use uuid::Uuid;

#[server]
pub async fn create_catering(catering_dto: CreateCateringDto) -> Result<Uuid, ServerFnError> {
    use chrono::TimeDelta;
    use leptos_axum::extract;
    use sqlx::postgres::types::PgInterval;
    use sqlx::postgres::PgPool;

    let dow: i16 = catering_dto
        .dow
        .into_iter()
        .enumerate()
        .map(|(i, d)| (d as i16) * 2_i16.pow(i as u32))
        .sum();

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let mut tr = pool.begin().await?;

    let grace_period = catering_dto.grace_period;

    if (catering_dto.until - catering_dto.since) < TimeDelta::days(1) {
        return Err(ServerFnError::new("Catering must last at least a day"));
    }

    let name = catering_dto.name.to_lowercase();
    let cat_name = name.trim();

    if cat_name.is_empty() {
        return Err(ServerFnError::new("Catering must have a name"));
    }

    let meals = catering_dto
        .meals
        .into_iter()
        .map(|meal| String::from(meal.to_lowercase().trim()))
        .collect::<Vec<_>>();

    sqlx::query!(
        "INSERT INTO meals (name) SELECT * FROM UNNEST($1::text[]) ON CONFLICT DO NOTHING",
        &meals
    )
    .execute(&mut *tr)
    .await?;

    let previous_catering = sqlx::query!("SELECT caterings.id FROM groups INNER JOIN caterings ON caterings.group_id = groups.id WHERE name = $1", cat_name).fetch_optional(&mut *tr).await?;

    if (!previous_catering.is_none()) {
        log!("Catering '{}' already exists", name);
        return Err(ServerFnError::new("Catering with this name already exists"));
    }

    let group_id = sqlx::query!(
        "INSERT INTO groups (name) VALUES ($1) RETURNING id",
        cat_name
    )
    .fetch_one(&mut *tr)
    .await?
    .id;

    let catering_id = sqlx::query!(
        "INSERT INTO caterings (group_id, grace_period, dow, since, until) VALUES ($1,$2,$3,$4,$5) RETURNING id",
        group_id,
        grace_period,
        dow,
        catering_dto.since,
        catering_dto.until
    )
    .fetch_one(&mut *tr)
    .await?.id;

    let meal_order = meals
        .iter()
        .enumerate()
        .map(|(i, _)| i as i32)
        .collect::<Vec<_>>();
    sqlx::query!("INSERT INTO catering_meals (catering_id, meal_id, meal_order) SELECT $1, meals.id, m_order FROM UNNEST($2::text[],$3::integer[]) as meal_names(name,m_order) INNER JOIN meals ON meals.name = meal_names.name", catering_id, &meals, &meal_order)
        .execute(&mut *tr).await?;

    sqlx::query!(
        "INSERT INTO group_relations (child,parent,level) VALUES ($1,$1,0)",
        group_id
    )
    .execute(&mut *tr)
    .await?;

    tr.commit().await?;

    Ok(catering_id)
}

#[server]
pub async fn get_caterings() -> Result<Vec<CateringDto>, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let caterings = sqlx::query!("SELECT caterings.id, groups.name FROM caterings INNER JOIN groups ON groups.id = caterings.group_id").fetch_all(&pool).await?.into_iter().map(|row| CateringDto{id: row.id, name: row.name}).collect();
    Ok(caterings)
}
