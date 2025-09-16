use leptos::logging::log;
use leptos::prelude::*;
use uuid::Uuid;

use crate::dtos::{
    catering::{AllergyDto, GuardianDetailDto, GuardianDto, MealDto},
    details::StudentDetailsDto,
    student::{CreateGuardianDto, CreateStudentDto, StudentDto, StudentInfoDto},
};

#[server]
pub async fn create_student(student: CreateStudentDto) -> Result<Uuid, ServerFnError> {
    use leptos_axum::extract;
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    let name = String::from(student.name.trim());
    let surname = String::from(student.surname.trim());

    let allergies = student
        .allergies
        .into_iter()
        .map(|a| String::from(a.trim()))
        .collect::<Vec<_>>();

    sqlx::query!(
        "INSERT INTO allergies (name) SELECT * FROM UNNEST($1::text[]) ON CONFLICT DO NOTHING",
        &allergies
    )
    .execute(&mut *tr)
    .await?;

    let allergy_combination_id = sqlx::query!("SELECT ac.id FROM allergy_combinations AS ac
        WHERE (SELECT COUNT(1) FROM allergy_combinations INNER JOIN allergies ON allergies.id = allergy_combinations.allergy_id WHERE allergy_combinations.id = ac.id) =
            (SELECT COUNT(1) FROM allergy_combinations INNER JOIN allergies ON allergies.id = allergy_combinations.allergy_id INNER JOIN UNNEST($1::text[]) AS names(name) ON names.name = allergies.name WHERE allergy_combinations.id = ac.id)", &allergies)
    .fetch_optional(&mut *tr).await?.map(|row| row.id);

    let allergy_combination_id = match allergy_combination_id {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4();
            sqlx::query!("INSERT INTO allergy_combinations (allergy_id,id) SELECT allergies.id, $2 FROM allergies INNER JOIN UNNEST($1::text[]) AS names(name) on allergies.name = names.name", &allergies, id).execute(&mut *tr).await?;
            id
        }
    };

    let guardian_names = student
        .guardians
        .into_iter()
        .map(|g| String::from(g.to_lowercase().trim()))
        .filter(|g| !g.is_empty())
        .collect::<Vec<_>>();

    sqlx::query!(
        "INSERT INTO guardians (fullname) SELECT * FROM UNNEST($1::text[]) ON CONFLICT DO NOTHING",
        &guardian_names
    )
    .execute(&mut *tr)
    .await?;

    let guardian_ids = sqlx::query!(
        "SELECT id FROM UNNEST($1::text[]) AS input(fullname) INNER JOIN  guardians ON guardians.fullname = input.fullname",
        &guardian_names
    )
    .fetch_all(&mut *tr)
    .await?
        .into_iter()
    .map(|row| row.id).collect::<Vec<_>>();

    let is_group = sqlx::query!(
        "SELECT groups.id FROM groups 
        WHERE groups.id = $1 AND NOT EXISTS (SELECT * FROM group_relations INNER JOIN groups AS gr ON gr.id = group_relations.child AND group_relations.parent = groups.id AND group_relations.level = 1)",
        student.group_id
    )
    .fetch_optional(&mut *tr)
    .await?;

    if is_group.is_none() {
        return Err(ServerFnError::new("Invalid group selected"));
    }

    let student_id = sqlx::query!("INSERT INTO students (name, surname, allergy_combination_id) VALUES ($1,$2,$3) RETURNING id", name, surname, allergy_combination_id).fetch_one(&mut *tr).await?.id;

    sqlx::query!("INSERT INTO student_guardians (student_id, guardian_id) SELECT $1, * FROM UNNEST($2::uuid[])", student_id, &guardian_ids).execute(&mut*tr).await?;

    sqlx::query!("INSERT INTO group_relations (child,parent,level) SELECT $1,parent,level + 1 FROM group_relations WHERE child=$2 UNION SELECT $1::uuid,$1::uuid,0", student_id, student.group_id).execute(&mut *tr).await?;

    sqlx::query!("INSERT INTO attendance (cause_id, target, day, meal_id, value) 
SELECT $2, $1, day, meal_id, true FROM caterings 
INNER JOIN group_relations ON group_relations.parent = caterings.group_id AND group_relations.child = $1
INNER JOIN generate_series(caterings.since, caterings.until, '1 day') as days(day) ON ((caterings.dow >> (EXTRACT(DOW FROM day)::smallint + 6) % 7 )&1) = 1
INNER JOIN catering_meals ON catering_meals.catering_id = caterings.id
", student_id, Uuid::new_v4()).execute(&mut*tr).await?;

    tr.commit().await?;

    Ok(student_id)
}

#[server]
pub async fn get_meals() -> Result<Vec<MealDto>, ServerFnError> {
    use leptos_axum::extract;
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let meals = sqlx::query!("SELECT id,name FROM meals")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| MealDto {
            id: row.id,
            name: row.name,
        })
        .collect();
    Ok(meals)
}

#[server]
pub async fn get_allergies() -> Result<Vec<AllergyDto>, ServerFnError> {
    use leptos_axum::extract;
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let allergies = sqlx::query!("SELECT id,name FROM allergies")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| AllergyDto {
            id: row.id,
            name: row.name,
        })
        .collect();
    Ok(allergies)
}

#[server]
pub async fn get_guardians() -> Result<Vec<GuardianDto>, ServerFnError> {
    use leptos_axum::extract;
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let guardians = sqlx::query!("SELECT * FROM guardians")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| GuardianDto {
            id: row.id,
            fullname: row.fullname,
        })
        .collect();
    Ok(guardians)
}

#[server]
pub async fn get_students() -> Result<Vec<StudentDto>, ServerFnError> {
    use leptos_axum::extract;
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let students = sqlx::query!("SELECT id,name,surname,parent as group_id FROM students INNER JOIN group_relations ON child=id AND level=1 WHERE removed= false")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| StudentDto {
            id: row.id,
            name: row.name,
            surname: row.surname,
            group_id: row.group_id,
        })
        .collect();
    Ok(students)
}

#[server]
pub async fn update_student(dto: StudentDetailsDto) -> Result<(), ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    if dto.guardians.is_empty() {
        return Err(ServerFnError::new(
            "Student needs to have at least one guardian",
        ));
    }

    let allergies = dto.allergies.iter().map(|a| a.id).collect::<Vec<_>>();
    let allergy_names = dto
        .allergies
        .iter()
        .map(|a| a.name.clone())
        .collect::<Vec<_>>();

    let allergy_combination = sqlx::query!("WITH combinations AS (SELECT id, ARRAY_AGG(allergy_id) AS al_id FROM allergy_combinations GROUP BY allergy_combinations.id)
            SELECT id FROM combinations
            WHERE $1::uuid[] @> combinations.al_id AND $1::uuid[] <@ combinations.al_id
", &allergies).fetch_optional(&mut *tr).await?.map(|row| row.id);

    sqlx::query!("INSERT INTO allergies (id,name) SELECT * FROM UNNEST($1::uuid[], $2::text[]) ON CONFLICT DO NOTHING", &allergies, &allergy_names).execute(&mut*tr).await?;

    let allergy_combination = match allergy_combination {
        Some(a) => a,
        None => {
            let id = Uuid::new_v4();
            sqlx::query!("INSERT INTO allergy_combinations (id, allergy_id) SELECT $1, * FROM UNNEST($2::uuid[])", id,
            &allergies).execute(&mut *tr).await?;
            id
        }
    };

    sqlx::query!("DELETE from student_guardians WHERE student_id=$1", dto.id)
        .execute(&mut *tr)
        .await?;

    let guardian_ids = dto.guardians.iter().map(|g| g.id).collect::<Vec<_>>();
    let guardian_names = dto
        .guardians
        .iter()
        .map(|g| g.fullname.clone())
        .collect::<Vec<_>>();

    sqlx::query!("INSERT INTO guardians (id, fullname) SELECT * FROM UNNEST($1::uuid[], $2::text[]) ON CONFLICT DO NOTHING", &guardian_ids, &guardian_names
    ).execute(&mut *tr).await?;

    sqlx::query!("INSERT INTO student_guardians (student_id, guardian_id) SELECT $1,id FROM UNNEST($2::uuid[]) AS guard(name) INNER JOIN guardians ON guardians.id = guard.name", dto.id, &guardian_ids).execute(&mut *tr).await?;

    sqlx::query!(
        "UPDATE students SET name=$2, surname=$3, allergy_combination_id=$4 WHERE id = $1",
        dto.id,
        dto.name,
        dto.surname,
        allergy_combination
    )
    .execute(&mut *tr)
    .await?;

    tr.commit().await?;
    Ok(())
}

#[server]
pub async fn create_guardian(dto: CreateGuardianDto) -> Result<(), ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    let fullname = dto.fullname.trim();

    if fullname.is_empty() {
        return Err(ServerFnError::new("Guardian must have name"));
    }

    let guardian_id = sqlx::query!(
        "INSERT INTO guardians (fullname, phone) VALUES ($1,$2) RETURNING id",
        fullname,
        dto.phone
    )
    .fetch_one(&mut *tr)
    .await?
    .id;

    let result = sqlx::query!("INSERT INTO student_guardians (student_id, guardian_id) SELECT *,$1 FROM UNNEST($2::uuid[])", guardian_id, &dto.students).execute(&mut *tr).await?;

    if result.rows_affected() as usize != dto.students.len() {
        return Err(ServerFnError::new(
            "Some students couldn't be assigned to guardian, check if all values are valid",
        ));
    }

    tr.commit().await?;
    Ok(())
}
