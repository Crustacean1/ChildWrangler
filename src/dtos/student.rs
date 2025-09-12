use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateStudentDto {
    pub name: String,
    pub group_id: Uuid,
    pub surname: String,
    pub allergies: Vec<String>,
    pub guardians: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StudentDto {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
    pub group_id: Uuid,
}
