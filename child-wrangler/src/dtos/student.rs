use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dtos::catering::{AllergyDto, GuardianDto};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateStudentDto {
    pub name: String,
    pub group_id: Uuid,
    pub surname: String,
    #[serde(default)]
    pub allergies: Vec<String>,
    #[serde(default)]
    pub guardians: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StudentDto {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
    pub group_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StudentInfoDto {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
    pub group_id: Uuid,
    pub guardians: Vec<GuardianDto>,
    pub allergies: Vec<AllergyDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateGuardianDto {
    pub fullname: String,
    pub phone: String,
    pub students: Vec<Uuid>,
}
