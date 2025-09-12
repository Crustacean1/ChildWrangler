use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::group::GroupDto;

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Breadcrumb {
    pub trail: Vec<GroupDto>,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct StudentDetailsDto {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
    pub guardian: String,
    pub guardian_id: Uuid,
    pub allergies: Vec<String>,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct GroupDetailsDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub enum EntityDto {
    Student(StudentDetailsDto),
    Group(GroupDetailsDto),
    LeafGroup(GroupDetailsDto),
    Catering(GroupDetailsDto),
}
