use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct GroupDto {
    pub id: Uuid,
    pub name: String,
    pub parent: Option<Uuid>,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct CreateGroupDto {
    pub name: String,
    pub parent: Uuid,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct SearchTerm {
    pub id: Uuid,
    pub name: String,
    pub parent_name: Option<String>,
}
