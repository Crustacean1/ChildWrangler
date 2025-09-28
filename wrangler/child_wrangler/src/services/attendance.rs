use chrono::{Days, Months, NaiveDate};
use dto::attendance::{
    AttendanceBreakdownDto, AttendanceHistoryDto, AttendanceHistoryItemDto, AttendanceItemDto,
    CateringMealDto, EffectiveAttendance, EffectiveMonthAttendance, GetAttendanceBreakdownDto,
    GetAttendanceHistoryDto, GetEffectiveMonthAttendance, GetMonthAttendanceDto, MealStatus,
    MonthAttendanceDto, UpdateAttendanceDto,
};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

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

#[server]
pub async fn get_effective_attendance(
    dto: GetEffectiveMonthAttendance,
) -> Result<EffectiveMonthAttendance, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let start = NaiveDate::from_ymd_opt(dto.year, dto.month, 1);
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

    let attendance = sqlx::query!(
        "SELECT DISTINCT ON (day,meal_id) day, meal_id, value, target, cause_id, attendance_override.id AS \"o_id: Option<Uuid>\" FROM effective_attendance 
            INNER JOIN group_relations ON group_relations.parent = effective_attendance.target
            LEFT JOIN attendance_override ON attendance_override.id = effective_attendance.cause_id
            LEFT JOIN processing_trigger ON processing_trigger.processing_id = cause_id
            WHERE group_relations.child=$1 AND (value = false OR level = 0) AND day >= $2 AND day < $3
            ORDER BY day, meal_id, level DESC
",
        dto.target,
        start,
        end
    )
    .fetch_all(&pool)
    .await?;

    let mut entries = BTreeMap::new();

    for entry in attendance {
        if let Some(day) = entry.day {
            entries.entry(day).or_insert(BTreeMap::new()).insert(
                entry.meal_id.unwrap_or(Default::default()),
                if entry.o_id.is_some() {
                    if entry.value.unwrap() {
                        EffectiveAttendance::Present
                    } else {
                        if entry.target == Some(dto.target) {
                            EffectiveAttendance::Absent
                        } else {
                            EffectiveAttendance::Blocked
                        }
                    }
                } else {
                    EffectiveAttendance::Present
                },
            );
        }
    }

    Ok(EffectiveMonthAttendance {
        attendance: entries,
    })
}

#[server]
pub async fn update_attendance(dto: UpdateAttendanceDto) -> Result<(), ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    let override_id = sqlx::query!(
        "INSERT INTO attendance_override (note) VALUES ($1) RETURNING id",
        dto.note,
    )
    .fetch_one(&mut *tr)
    .await?
    .id;

    sqlx::query!("INSERT INTO attendance (cause_id,target,day,meal_id,value) SELECT $1,$2,day,meal_id, false FROM UNNEST($3::date[]) AS arg1(day) 
                    CROSS JOIN UNNEST($4::uuid[]) AS arg2(meal_id)", override_id, dto.target, &dto.days, &dto.inactive_meals).execute(&mut *tr).await?;

    sqlx::query!("INSERT INTO attendance (cause_id,target,day,meal_id,value) SELECT $1,$2,day,meal_id, true FROM UNNEST($3::date[]) AS arg1(day) 
                    CROSS JOIN UNNEST($4::uuid[]) AS arg2(meal_id)", override_id, dto.target, &dto.days, &dto.active_meals).execute(&mut *tr).await?;

    tr.commit().await?;
    Ok(())
}

#[server]
pub async fn get_attendance_history(
    dto: GetAttendanceHistoryDto,
) -> Result<AttendanceHistoryDto, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let history = sqlx::query!(
        "SELECT attendance.originated, attendance.value , attendance_override.note AS note, processing_trigger.message_id AS msg_id FROM attendance 
        LEFT JOIN attendance_override ON attendance_override.id = attendance.cause_id
        LEFT JOIN processing_trigger ON processing_trigger.processing_id = attendance.cause_id
        WHERE target=$1 AND day = $2 AND meal_id = $3 ORDER BY originated",
        dto.target,
        dto.date,
        dto.meal_id
    )
    .fetch_all(&pool)
    .await?;

    let history = history
        .into_iter()
        .map(|row| {
            if let Some(msg_id) = row.msg_id {
                AttendanceHistoryItemDto {
                    time: row.originated,
                    item: AttendanceItemDto::Cancellation(msg_id, String::new(), String::new()),
                }
            } else if let Some(note) = row.note {
                AttendanceHistoryItemDto {
                    time: row.originated,
                    item: AttendanceItemDto::Override(note, false),
                }
            } else {
                AttendanceHistoryItemDto {
                    time: row.originated,
                    item: AttendanceItemDto::Init,
                }
            }
        })
        .collect::<Vec<_>>();

    let events = sqlx::query!("SELECT note, processing_id, target, level FROM group_relations
    INNER JOIN effective_attendance ON effective_attendance.target = group_relations.parent
    LEFT JOIN attendance_override ON attendance_override.id = effective_attendance.cause_id
    LEFT JOIN processing_trigger ON processing_trigger.processing_id = effective_attendance.cause_id
    WHERE group_relations.child = $1 AND effective_attendance.day = $2 AND effective_attendance.meal_id = $3 AND ((level > 0 AND value = false) OR level = 0)
    ORDER BY level DESC LIMIT 1", dto.target, dto.date, dto.meal_id)
    .fetch_optional(&pool)
    .await?;

    log!("Wtf: {:?}", events);
    Ok(AttendanceHistoryDto {
        events: history,
        status: if let Some(events) = events {
            if events.level != 0 {
                MealStatus::Blocked(events.target.unwrap_or(Uuid::nil()))
            } else if events.note.is_some() {
                MealStatus::Overriden
            } else if events.processing_id.is_some() {
                MealStatus::Cancelled
            } else {
                MealStatus::Init
            }
        } else {
            MealStatus::Init
        },
    })
}

#[server]
pub async fn get_attendance_breakdown(
    dto: GetAttendanceBreakdownDto,
) -> Result<AttendanceBreakdownDto, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let meal = sqlx::query!("SELECT * FROM meals WHERE meals.id = $1", dto.meal_id)
        .fetch_one(&pool)
        .await?
        .name;

    let attendance = sqlx::query!("SELECT group_relations.child AS id, SUM(rooted_attendance.present::int) AS attendance FROM group_relations
    INNER JOIN rooted_attendance ON rooted_attendance.root = group_relations.child AND rooted_attendance.day = $2 AND rooted_attendance.meal_id = $3
    WHERE group_relations.parent=$1 AND group_relations.level = 1
    GROUP BY group_relations.child", dto.target, dto.date, dto.meal_id)
        .fetch_all(&pool).await?;

    let names = sqlx::query!(
        "SELECT COALESCE(groups.name, FORMAT('%s %s', students.name, students.surname)) AS name, COALESCE(groups.id,students.id) AS id FROM group_relations
    LEFT JOIN students ON students.id = group_relations.child
    LEFT JOIN groups ON groups.id = group_relations.child
    WHERE group_relations.parent = $1",
        dto.target
    )
    .fetch_all(&pool)
    .await?.into_iter().filter_map(|row| {

Some((row.id?,row.name?))
        })
        .collect::<HashMap<_,_>>();

    let attendance = attendance
        .into_iter()
        .filter_map(|row| {
            Some((
                row.id,
                (names.get(&row.id)?.clone(), row.attendance.unwrap_or(0)),
            ))
        })
        .collect();

    Ok(AttendanceBreakdownDto { attendance, meal })
}
