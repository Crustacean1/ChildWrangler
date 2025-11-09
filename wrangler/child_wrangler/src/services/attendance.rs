use chrono::{Days, Months, NaiveDate};
use dto::{
    attendance::{
        AttendanceBreakdownDto, AttendanceHistoryDto, AttendanceHistoryItemDto, AttendanceItemDto,
        AttendanceOverviewDto, AttendanceOverviewType, AttendanceStatus, CateringMealDto,
        EffectiveAttendance, EffectiveMonthAttendance, GetAttendanceBreakdownDto,
        GetAttendanceHistoryDto, GetEffectiveMonthAttendance, GetMonthAttendanceDto, MealStatus,
        MonthAttendanceDto, MonthlyStudentAttendanceDto, UpdateAttendanceDto,
    },
    catering::MealDto,
    group::GroupDto,
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

    let is_student = sqlx::query!(
        "SELECT id FROM students WHERE students.id = $1 LIMIT 1",
        dto.target
    )
    .fetch_optional(&pool)
    .await?
    .is_some();

    let attendance = sqlx::query!(
        "SELECT DISTINCT ON (day,meal_id) day, meal_id, value, target, effective_attendance.cause_id, 
    (attendance_override.id IS NOT NULL) AS is_override,
    (messages.id IS NOT NULL) AS is_cancellation FROM effective_attendance 
    INNER JOIN group_relations ON group_relations.parent = effective_attendance.target
    LEFT JOIN attendance_override ON attendance_override.id = effective_attendance.cause_id
    LEFT JOIN messages ON messages.id = effective_attendance.cause_id
    WHERE group_relations.child=$1 AND day >= $2 AND day < $3
    ORDER BY day, meal_id, value, level DESC",
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
                if entry.value.unwrap_or(false) {
                    EffectiveAttendance::Present
                } else {
                    if entry.is_override.unwrap_or(false) {
                        if Some(dto.target) == entry.target {
                            EffectiveAttendance::Absent
                        } else {
                            EffectiveAttendance::Blocked
                        }
                    } else if entry.is_cancellation.unwrap_or(false) {
                        EffectiveAttendance::Cancelled
                    } else {
                        panic!("Wtf? Invalid attendance record, consult administrator")
                    }
                },
            );
        }
    }

    Ok(EffectiveMonthAttendance {
        is_student,
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
        r#"SELECT attendance.target, attendance.cause_id, messages.phone AS "phone?", messages.content AS "content?", originated, ARRAY_AGG((meal_id, value)) AS "meals: Vec<(Uuid,bool)>", note, messages.id AS "msg_id?" FROM group_relations
    INNER JOIN attendance ON attendance.target = group_relations.parent
    LEFT JOIN attendance_override ON attendance_override.id = attendance.cause_id
    LEFT JOIN messages ON messages.id = attendance.cause_id
    WHERE group_relations.child = $1 AND attendance.day = $2 
    GROUP BY attendance.cause_id, attendance.originated, attendance_override.id, messages.id , target
    ORDER BY originated"#,
        dto.target,
        dto.date
    )
    .fetch_all(&pool)
    .await?;

    let history = history
        .into_iter()
        .map(|row| {
            if let Some(msg_id) = row.msg_id {
                AttendanceHistoryItemDto {
                    time: row.originated,
                    meals: row.meals.unwrap_or_default(),
                    item: AttendanceItemDto::Cancellation(msg_id, row.phone.unwrap_or_default(), row.content.unwrap_or_default()),
                }
            } else if let Some(note) = row.note {
                AttendanceHistoryItemDto {
                    time: row.originated,
                    meals: row.meals.unwrap_or_default(),
                    item: AttendanceItemDto::Override(row.target, note),
                }
            } else {
                AttendanceHistoryItemDto {
                    time: row.originated,
                    meals: row.meals.unwrap_or_default(),
                    item: AttendanceItemDto::Init,
                }
            }
        })
        .collect::<Vec<_>>();

    let events = sqlx::query!(r#"SELECT note, messages.id AS "trigger_id?", target, level FROM group_relations
    INNER JOIN effective_attendance ON effective_attendance.target = group_relations.parent
    LEFT JOIN attendance_override ON attendance_override.id = effective_attendance.cause_id
    LEFT JOIN messages ON messages.id = effective_attendance.cause_id
    WHERE group_relations.child = $1 AND effective_attendance.day = $2 AND effective_attendance.meal_id = $3 AND ((level > 0 AND value = false) OR level = 0)
    ORDER BY level DESC LIMIT 1"#, dto.target, dto.date, Uuid::nil())
    .fetch_optional(&pool)
    .await?;

    Ok(AttendanceHistoryDto {
        events: history,
        status: if let Some(events) = events {
            if events.level != 0 {
                MealStatus::Blocked(events.target.unwrap_or(Uuid::nil()))
            } else if events.note.is_some() {
                MealStatus::Overriden
            } else if events.trigger_id.is_some() {
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

    let meals = sqlx::query!(
        "SELECT meals.id, meals.name FROM meals
    INNER JOIN catering_meals ON catering_meals.meal_id = meals.id
    INNER JOIN caterings ON caterings.id = catering_meals.catering_id
    INNER JOIN group_relations ON group_relations.parent = caterings.group_id
    WHERE group_relations.child = $1",
        dto.target
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| MealDto {
        id: row.id,
        name: row.name,
    })
    .collect();

    let groups: Vec<GroupDto> = {
        let groups: Vec<_> = sqlx::query!(
            "SELECT groups.id, groups.name FROM groups
    INNER JOIN group_relations ON group_relations.child = groups.id AND group_relations.level = 1
    WHERE group_relations.parent = $1 AND NOT groups.removed ORDER BY name",
            dto.target
        )
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| GroupDto {
            id: row.id,
            name: row.name,
            parent: None,
        })
        .collect();

        if groups.is_empty() {
            sqlx::query!("SELECT students.id, students.name, students.surname FROM students
        INNER JOIN group_relations ON group_relations.child = students.id AND group_relations.level = 1
        WHERE group_relations.parent = $1 AND NOT students.removed ORDER BY surname", dto.target).fetch_all(&pool).await?.into_iter().map(|row| GroupDto{
                    id: row.id,
                    name: format!("{} {}", row.name, row.surname),
                    parent: None
                }).collect()
        } else {
            groups
        }
    };

    let attendance_rows = sqlx::query!("SELECT group_relations.child AS id, meal_id, COUNT(*) AS max_attendance, SUM(present::int) AS attendance FROM group_relations
    INNER JOIN rooted_attendance ON rooted_attendance.root = group_relations.child
    WHERE group_relations.parent = $1 AND group_relations.level = 1 AND rooted_attendance.day = $2
    GROUP BY group_relations.child, rooted_attendance.meal_id
    ", dto.target, dto.date).fetch_all(&pool)
    .await?;

    let mut attendance = HashMap::new();

    for row in attendance_rows {
        attendance.entry(row.id).or_insert(HashMap::new()).insert(
            row.meal_id.unwrap_or_default(),
            (row.max_attendance.unwrap_or(0), row.attendance.unwrap_or(0)),
        );
    }

    Ok(AttendanceBreakdownDto {
        attendance,
        meals,
        groups,
    })
}

#[server]
pub async fn get_monthly_summary(
    target: Uuid,
    year: i32,
    month: u32,
) -> Result<String, ServerFnError> {
    use sqlx::postgres::PgPool;
    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    use csv::WriterBuilder;

    let start = NaiveDate::from_ymd_opt(year, month, 1).ok_or(ServerFnError::new(
        "Failed to construct start date from provided arguments",
    ))?;
    let end = start
        .checked_add_months(Months::new(1))
        .ok_or(ServerFnError::new(
            "Failed to construct end date from provided arguments",
        ))?;

    let catering_group_id = sqlx::query!(
        "SELECT group_id FROM group_relations
    INNER JOIN caterings ON caterings.group_id = group_relations.parent
    WHERE group_relations.child = $1",
        target
    )
    .fetch_one(&pool)
    .await?
    .group_id;

    let attendance = sqlx::query!(
        "WITH student_attendance AS (SELECT student_id, SUM(present::int) AS student_attendance FROM rooted_attendance 
    WHERE root = $1 AND day >= $2 AND day < $3
    GROUP BY student_id)
    SELECT groups.name AS group_name, students.name, students.surname, students.id, student_attendance AS attendance FROM student_attendance
    INNER JOIN students ON student_attendance.student_id = students.id
    INNER JOIN group_relations AS direct_relation ON direct_relation.level = 1 AND direct_relation.child = students.id
    INNER JOIN groups ON groups.id = direct_relation.parent",catering_group_id, start, end)
        .fetch_all(&pool)
        .await?
    .into_iter()
        .map(|row| MonthlyStudentAttendanceDto{
            student_id: row.id,
            name: row.name,
            surname: row.surname,
            attendance: row.attendance.unwrap_or(0) as u32,
            group: row.group_name,
        }).collect::<Vec<_>>();

    let mut wrtr = WriterBuilder::new().from_writer(vec![]);

    for student in attendance {
        wrtr.serialize(student)?;
    }

    wrtr.flush()?;

    Ok(String::from_utf8(wrtr.into_inner()?)?)
}

#[server]
pub async fn get_attendance_overview(
    date: NaiveDate,
    catering_id: Uuid,
) -> Result<AttendanceOverviewDto, ServerFnError> {
    use sqlx::postgres::PgPool;
    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let students = sqlx::query!("SELECT bool_and(value) AS value, meal_id,  (attendance_override.id IS NOT NULL) AS is_override, students.id AS id, (messages.id IS NOT NULL) AS is_cancellation, students.allergy_combination_id AS allergies_id FROM caterings
    INNER JOIN group_relations ON group_relations.parent = caterings.group_id
    INNER JOIN students ON students.id = group_relations.child
    INNER JOIN total_attendance ON total_attendance.student_id = students.id
    LEFT JOIN attendance_override ON attendance_override.id = total_attendance.cause_id
    LEFT JOIN messages ON messages.id = total_attendance.cause_id
    WHERE total_attendance.day = $1 AND caterings.id = $2
    GROUP BY students.id, meal_id, is_override, is_cancellation
",  date, catering_id)
    .fetch_all(&pool)
    .await?;

    let mut attendance = HashMap::new();
    let mut student_list = HashMap::new();

    for student in students {
        let meal_id = student.meal_id.unwrap();

        if student.value.unwrap_or(false) {
            *attendance
                .entry(meal_id)
                .or_insert(HashMap::new())
                .entry(AttendanceOverviewType::Present(
                    student.allergies_id.unwrap_or_default(),
                ))
                .or_insert(0) += 1;
        } else if student.is_override.unwrap_or(false) {
            *attendance
                .entry(meal_id)
                .or_insert(HashMap::new())
                .entry(AttendanceOverviewType::Disabled)
                .or_insert(0) += 1;
        } else if student.is_cancellation.unwrap_or(false) {
            *attendance
                .entry(meal_id)
                .or_insert(HashMap::new())
                .entry(AttendanceOverviewType::Cancelled)
                .or_insert(0) += 1;
        }

        student_list
            .entry(student.id)
            .or_insert(HashMap::new())
            .insert(
                meal_id,
                if student.value.unwrap_or(false) {
                    AttendanceStatus::Present
                } else if student.is_override.unwrap_or(false) {
                    AttendanceStatus::Overriden
                } else if student.is_cancellation.unwrap_or(false) {
                    AttendanceStatus::Cancelled
                } else {
                    panic!("This shouldn't have happened: invalid student presence")
                },
            );
    }
    let attendance = attendance
        .into_iter()
        .map(|(id, types)| (id, types.into_iter().collect()))
        .collect();

    Ok(AttendanceOverviewDto {
        student_list,
        attendance,
    })
}
