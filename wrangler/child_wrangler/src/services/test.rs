use std::collections::HashSet;

use chrono::{Days, NaiveTime, Utc};
use dto::catering::{CateringDto, CreateCateringDto};
use dto::group::CreateGroupDto;
use dto::guardian::GuardianDetailDto;
use dto::student::CreateStudentDto;
use leptos::logging::log;
use leptos::prelude::*;
use uuid::Uuid;

use crate::components::modals::modify_guardian_phone;
use crate::services::catering::create_catering;
use crate::services::group::{create_group, get_details, get_groups};
use crate::services::messages::update_guardian;
use crate::services::student::create_student;

#[server]
pub async fn generate_random_data(
    catering_count: i32,
    group_count: i32,
    student_count: i32,
    guardian_count: i32,
) -> Result<(), ServerFnError> {
    use fake::faker::company::raw::{CompanyName, Industry};
    use fake::faker::name::raw::{FirstName, LastName, Name};
    use fake::faker::phone_number::raw::PhoneNumber;
    use fake::locales::EN;
    use fake::Fake;
    use rand::seq::IndexedRandom;

    let now = Utc::now().date_naive();

    let meals = vec![
        String::from("Śniadanie"),
        String::from("Obiad"),
        String::from("Podwieczorek"),
        String::from("Kolacja"),
    ];

    log!("Starting random generation");

    for i in 0..catering_count {
        log!("Generating catering");
        create_catering(CreateCateringDto {
            name: format!("Catering {}", i),
            since: now.checked_sub_days(Days::new((1..200).fake())).unwrap(),
            until: now.checked_add_days(Days::new((1..200).fake())).unwrap(),
            grace_period: NaiveTime::from_hms_milli_opt(
                (0..24).fake(),
                (0..60).fake(),
                (0..60).fake(),
                (0..1000).fake(),
            )
            .unwrap(),
            meals: meals
                .clone()
                .into_iter()
                .take((1..meals.len()).fake::<usize>())
                .collect(),
            dow: (0..7).map(|m| (0..2).fake::<u32>() == 1).collect(),
        })
        .await?;
    }

    let mut used_groups = HashSet::new();
    for i in 0..group_count {
        let groups = get_groups().await?;
        log!("Generating group");
        let id = groups[(0..groups.len()).fake::<usize>()].id;
        used_groups.insert(id);

        create_group(CreateGroupDto {
            name: format!("Group {}", i),
            parent: id,
        })
        .await?;
    }

    let groups = get_groups()
        .await?
        .into_iter()
        .map(|g| g.id)
        .collect::<HashSet<_>>();

    let groups = groups.difference(&used_groups).collect::<Vec<_>>();

    let mut guardian_list = (0..student_count).map(|_| {
        format!(
            "{} {}",
            FirstName(EN).fake::<String>(),
            LastName(EN).fake::<String>()
        )
    });

    for i in 0..student_count {
        let group_id = *groups[(..groups.len()).fake::<usize>()];
        log!("Generating student group id {}", group_id);
        let mut guardians = vec![];
        for i in 0..((0..3).fake::<usize>()) {
            if let Some(g) = guardian_list.next() {
                guardians.push(g);
            }
        }

        let student_id = create_student(CreateStudentDto {
            name: FirstName(EN).fake(),
            group_id,
            surname: LastName(EN).fake(),
            allergies: vec![],
            guardians,
        })
        .await?;

        match get_details(student_id).await? {
            dto::details::EntityDto::Student(dto) => {
                for g in dto.guardians {
                    update_guardian(GuardianDetailDto {
                        fullname: g.fullname,
                        id: g.id,
                        phone: Some(PhoneNumber(EN).fake::<String>()),
                        students: vec![],
                    })
                    .await?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
