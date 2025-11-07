use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::student::StudentDto;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianDto {
    pub id: Uuid,
    pub fullname: String,
    pub phone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianDetailDto {
    pub id: Uuid,
    pub fullname: String,
    pub phone: Option<String>,
    pub students: Vec<StudentDto>,
}
