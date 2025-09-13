use leptos::logging::log;
use leptos::prelude::*;
use uuid::Uuid;

use crate::dtos::{
    catering::{AllergyDto, GuardianDto},
    details::{EntityDto, GroupDetailsDto, StudentDetailsDto},
    group::{CreateGroupDto, GroupDto, GroupInfoDto, ModifyGroupDto, SearchTerm},
};

#[server]
pub async fn create_group(group: CreateGroupDto) -> Result<Uuid, ServerFnError> {
    use sqlx::postgres::PgPool;
    use uuid::Uuid;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    let is_group = sqlx::query!(
        "SELECT groups.id FROM groups 
        WHERE groups.id = $1 AND NOT EXISTS (SELECT * FROM group_relations INNER JOIN students ON students.id = group_relations.child AND group_relations.parent = groups.id AND group_relations.level = 1)",
        group.parent
    )
    .fetch_optional(&mut *tr)
    .await?;

    if is_group.is_none() {
        log!(
            "Group {} has students, so it cannot also have groups",
            group.parent
        );
        return Err(ServerFnError::new("Invalid group selected"));
    }

    let name = String::from(group.name.to_lowercase().trim());
    let id: Uuid = sqlx::query!("INSERT INTO groups (name) VALUES ($1) RETURNING id", name)
        .fetch_one(&mut *tr)
        .await?
        .id;

    sqlx::query!("INSERT INTO group_relations (child,parent,level) SELECT $1,parent,level + 1 FROM group_relations WHERE child=$2 UNION SELECT $1::uuid,$1::uuid,0", id, group.parent).execute(&mut *tr).await?;

    log!("Created group {} with parent {}", id, group.parent);

    tr.commit().await?;
    Ok(id)
}

#[server]
pub async fn modify_group(dto: ModifyGroupDto) -> Result<(), ServerFnError> {
    use sqlx::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;
    let rows = sqlx::query!("UPDATE groups set name= $2 WHERE id=$1", dto.id, dto.name)
        .execute(&mut *tr)
        .await?
        .rows_affected();
    tr.commit().await?;

    Ok(())
}

#[server]
pub async fn get_groups() -> Result<Vec<GroupDto>, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let result = sqlx::query!("SELECT * FROM groups LEFT JOIN group_relations ON group_relations.child = groups.id AND group_relations.level = 1 WHERE NOT groups.removed")
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| GroupDto {
            id: row.id,
            name: row.name,
            parent: row.parent,
        })
        .collect();
    Ok(result)
}

#[server]
pub async fn transfer_group(transfer: (Uuid, Uuid)) -> Result<(), ServerFnError> {
    use sqlx::postgres::PgPool;

    let (child, new_parent) = transfer;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    let is_invalid_group = sqlx::query!(
        "SELECT group_relations.child FROM group_relations WHERE child = $1 AND parent = $2 LIMIT 1",
        new_parent,
        child
    )
    .fetch_optional(&mut *tr)
    .await?.is_some();

    if is_invalid_group {
        log!("The fuck: {} to {}", child, new_parent);
        return Err(ServerFnError::new(
            "Cannot move parent to subdirectory of child",
        ));
    }

    let is_student = sqlx::query!("SELECT * FROM students WHERE id = $1", child)
        .fetch_optional(&mut *tr)
        .await?
        .is_some();

    let is_group_node = sqlx::query!("SELECT * FROM group_relations INNER JOIN groups ON groups.id = group_relations.child WHERE parent=$1 AND level = 1 LIMIT 1", new_parent).fetch_optional(&mut *tr).await?.is_some();

    let is_student_node = sqlx::query!("SELECT * FROM group_relations INNER JOIN students ON students.id = group_relations.child WHERE parent=$1 AND level = 1 LIMIT 1", new_parent).fetch_optional(&mut *tr).await?.is_some();

    if (is_group_node && is_student) {
        return Err(ServerFnError::new(
            "Cannot add student to group which contains other groups",
        ));
    }

    if (is_student_node && !is_student) {
        return Err(ServerFnError::new(
            "Cannot add group to group which contains students",
        ));
    }

    sqlx::query!(
        "DELETE FROM group_relations AS a
                USING group_relations AS gr1 
                CROSS JOIN group_relations AS gr2 
                WHERE gr1.parent = $1 AND gr2.child = $1 AND gr2.level > 0 AND a.parent = gr2.parent AND a.child = gr1.child",
        child
    )
    .execute(&mut *tr)
    .await?;

    let new_entries = sqlx::query!(
        "INSERT INTO group_relations (child,parent,level) 
            SELECT gr2.child ,gr.parent, gr.level + gr2.level + 1 FROM group_relations  AS gr
            INNER JOIN group_relations AS gr2 ON gr2.parent=$1
            WHERE gr.child=$2",
        child,
        new_parent
    )
    .execute(&mut *tr)
    .await?;

    log!("New entries: {:?}", new_entries);

    tr.commit().await?;

    Ok(())
}

#[server]
pub async fn get_group_info(id: Uuid) -> Result<GroupInfoDto, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let info = sqlx::query!("SELECT * FROM groups WHERE id = $1", id)
        .fetch_one(&pool)
        .await?;

    let students = sqlx::query!(
        "SELECT count(*) AS student_count FROM groups 
                    INNER JOIN group_relations AS s_gr ON s_gr.parent = groups.id AND s_gr.level > 0
                    INNER JOIN students ON students.id = s_gr.child
                    WHERE students.removed = false
                    GROUP BY groups.id
                    HAVING groups.id = $1",
        id
    )
    .fetch_optional(&pool)
    .await?;

    let groups = sqlx::query!(
        "SELECT count(*) AS group_count FROM groups 
                    INNER JOIN group_relations AS s_gr ON s_gr.parent = groups.id AND s_gr.level > 0
                    INNER JOIN groups AS gr ON gr.id = s_gr.child
                    WHERE gr.removed = false
                    GROUP BY groups.id
                    HAVING groups.id = $1",
        id
    )
    .fetch_optional(&pool)
    .await?;

    Ok(GroupInfoDto {
        id: info.id,
        name: info.name,
        student_count: students.and_then(|s| s.student_count).unwrap_or(0),
        group_count: groups.and_then(|s| s.group_count).unwrap_or(0),
    })
}

#[server]
pub async fn delete_group(id: Uuid) -> Result<(), ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let mut tr = pool.begin().await?;

    let group_rows = sqlx::query!("UPDATE groups SET removed = true FROM groups AS gr JOIN group_relations ON group_relations.child = gr.id AND group_relations.parent = $1 WHERE groups.id = gr.id", id).execute(&mut *tr).await?.rows_affected();

    let student_rows = sqlx::query!("UPDATE students SET removed = true FROM students AS gr JOIN group_relations ON group_relations.child = gr.id AND group_relations.parent = $1 WHERE students.id = gr.id", id).execute(&mut *tr).await?.rows_affected();

    tr.commit().await?;

    log!(
        "Removed {} groups and {} students",
        group_rows,
        student_rows
    );

    Ok(())
}

#[server]
pub async fn get_breadcrumbs(id: Uuid) -> Result<Vec<GroupDto>, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;
    let results = sqlx::query!("SELECT groups.id, groups.name FROM group_relations INNER JOIN groups ON groups.id = group_relations.parent WHERE group_relations.child = $1 ORDER BY level DESC", id)
    .fetch_all(&pool).await?
        .into_iter().map(|row| GroupDto{id: row.id, name: row.name, parent: None});
    let student = sqlx::query!("SELECT * from students WHERE id = $1", id)
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|row| GroupDto {
            name: format!("{} {}", row.name, row.surname),
            id: row.id,
            parent: None,
        });
    return Ok(results.chain(student).collect());
}

#[server]
pub async fn get_search_terms() -> Result<Vec<SearchTerm>, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let group_terms = sqlx::query!(
        "SELECT groups.id, groups.name, parents.name AS parent_name FROM groups
    LEFT JOIN group_relations ON group_relations.child = groups.id AND group_relations.level = 1
    LEFT JOIN groups AS parents ON parents.id = group_relations.parent
    WHERE groups.removed=false"
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| SearchTerm {
        name: row.name,
        parent_name: row.parent_name,
        id: row.id,
    });

    let student_terms = sqlx::query!(
        "SELECT students.id, students.name, students.surname, parents.name AS parent_name FROM students
    LEFT JOIN group_relations ON group_relations.child = students.id AND group_relations.level = 1
    LEFT JOIN groups AS parents ON parents.id = group_relations.parent
    WHERE students.removed=false")
    .fetch_all(&pool)
    .await?
    .into_iter().map(|row| SearchTerm {
        name: format!("{} {}", row.name, row.surname),
        parent_name: row.parent_name,
        id: row.id,
    });

    Ok(group_terms.chain(student_terms).collect())
}

#[server]
pub async fn get_details(id: Uuid) -> Result<EntityDto, ServerFnError> {
    use sqlx::postgres::PgPool;

    let pool: PgPool = use_context().ok_or(ServerFnError::new("Failed to retrieve db pool"))?;

    let student = sqlx::query!("SELECT id,name, surname FROM students WHERE id = $1", id)
        .fetch_optional(&pool)
        .await?;

    if let Some(student) = student {
        let allergies = sqlx::query!(
            "SELECT allergies.* FROM allergies 
        INNER JOIN allergy_combinations ON allergy_combinations.allergy_id = allergies.id
        INNER JOIN students ON students.allergy_combination_id = allergy_combinations.id
        WHERE students.id = $1",
            id
        )
        .fetch_all(&pool)
        .await?;

        let guardians = sqlx::query!(
            "SELECT guardians.* FROM guardians INNER JOIN student_guardians ON student_id = $1 AND student_guardians.guardian_id = guardians.id",
            id
        )
        .fetch_all(&pool)
        .await?;

        Ok(EntityDto::Student(StudentDetailsDto {
            id,
            name: student.name,
            surname: student.surname,
            guardians: guardians
                .into_iter()
                .map(|g| GuardianDto {
                    id: g.id,
                    fullname: g.fullname,
                })
                .collect(),
            allergies: allergies
                .into_iter()
                .map(|row| AllergyDto {
                    name: row.name,
                    id: row.id,
                })
                .collect(),
        }))
    } else {
        let group = sqlx::query!("SELECT id, name, gr_parent.parent AS \"parent:Option<Uuid>\" FROM groups 
            LEFT JOIN group_relations AS gr_parent ON gr_parent.level = 1 AND gr_parent.child = groups.id WHERE groups.id = $1", id).fetch_optional(&pool).await?;
        if let Some(group) = group {
            let result = GroupDetailsDto {
                id: group.id,
                name: group.name,
            };

            if group.parent.is_none() {
                Ok(EntityDto::Catering(result))
            } else {
                let child_groups = sqlx::query!("SELECT * FROM groups INNER JOIN group_relations ON group_relations.level = 1 AND group_relations.parent = $1 LIMIT 1", id).fetch_optional(&pool).await?;
                if child_groups.is_some() {
                    let has_students = sqlx::query!("SELECT * FROM group_relations 
            INNER JOIN students ON students.id = group_relations.child AND group_relations.level = 1 
            WHERE group_relations.parent = $1 LIMIT 1", id).fetch_optional(&pool).await?;
                    if has_students.is_some() {
                        Ok(EntityDto::StudentGroup(result))
                    } else {
                        Ok(EntityDto::Group(result))
                    }
                } else {
                    Ok(EntityDto::LeafGroup(result))
                }
            }
        } else {
            log!("Entity with id {} has not been found", id);
            Err(ServerFnError::new("No such entity"))
        }
    }
}
