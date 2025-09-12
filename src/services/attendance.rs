use chrono::{Days, Months, NaiveDate};
use std::collections::{BTreeMap, HashMap};

use crate::dtos::attendance::{CateringMealDto, GetMonthAttendanceDto, MonthAttendanceDto};
use leptos::{logging::log, prelude::*};

#[server]
pub async fn get_month_attendance(
    dto: GetMonthAttendanceDto,
) -> Result<MonthAttendanceDto, ServerFnError> {
    use chrono::Datelike;
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let catering = sqlx::query!("SELECT * FROM caterings INNER JOIN group_relations ON group_relations.parent = caterings.group_id WHERE group_relations.child = $1", dto.target)
        .fetch_optional(&pool)
        .await?.ok_or(ServerFnError::new("No catering found for given id"))?;

    let start = NaiveDate::from_ymd_opt(dto.year as i32, dto.month, 1);
    let end = start.and_then(|date| date.checked_add_months(Months::new(1)));

    let (Some(start), Some(end)) = (start, end) else {
        log!(
            "Failed to parse the dates: {:?} {:?} for request: {:?}",
            start,
            end,
            dto
        );
        return Err(ServerFnError::new("Failed to parse provided date"));
    };

    let attendance = sqlx::query!("SELECT COUNT(*) as att, meal_id, day FROM rooted_attendance
                        WHERE rooted_attendance.root = $1 AND rooted_attendance.day >= $2 AND rooted_attendance.day < $3 AND present = true
                        GROUP BY meal_id, day", dto.target, start, end).fetch_all(&pool).await?;

    let meal_order = sqlx::query!("SELECT meals.id, meals.name FROM meals INNER JOIN catering_meals ON catering_meals.meal_id = meals.id WHERE catering_meals.catering_id = $1 ORDER BY meal_order", catering.id).fetch_all(&pool).await?;

    let dow = (0..7)
        .map(|i| (catering.dow >> i) & 1 == 1)
        .collect::<Vec<_>>();

    let mut days = std::iter::successors(Some(start), |x: &NaiveDate| {
        x.checked_add_days(Days::new(1))
            .and_then(|x| if x >= end { None } else { Some(x) })
    })
    .map(|x| (x, vec![]))
    .collect::<BTreeMap<_, _>>();

    for entry in attendance {
        entry.day.map(|date| {
            if let Some(day) = days.get_mut(&date) {
                if let Some(meal_id) = entry.meal_id {
                    day.push((meal_id, entry.att.unwrap_or(0) as u32));
                }
            }
        });
    }

    let days = days
        .into_iter()
        .map(|(key, value)| (key, value.into_iter().collect::<BTreeMap<_, _>>()))
        .collect::<BTreeMap<_, _>>();

    Ok(MonthAttendanceDto {
        meals: meal_order
            .into_iter()
            .map(|row| CateringMealDto {
                id: row.id,
                name: row.name,
            })
            .collect(),
        days_of_week: dow,
        start: catering.since,
        end: catering.until,
        attendance: days,
    })
}
