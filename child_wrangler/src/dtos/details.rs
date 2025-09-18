use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::{
    catering::{AllergyDto, GuardianDto},
    group::GroupDto,
};

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Breadcrumb {
    pub trail: Vec<GroupDto>,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct StudentDetailsDto {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
    #[serde(default)]
    pub guardians: Vec<GuardianDto>,
    #[serde(default)]
    pub allergies: Vec<AllergyDto>,
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
    StudentGroup(GroupDetailsDto),
    LeafGroup(GroupDetailsDto),
    Catering(GroupDetailsDto),
}
