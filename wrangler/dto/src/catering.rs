use chrono::{NaiveDate, NaiveTime, TimeDelta};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateCateringDto {
    pub name: String,
    pub since: NaiveDate,
    pub until: NaiveDate,
    pub grace_period: NaiveTime,
    pub meals: Vec<String>,
    pub dow: Vec<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MealDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllergyDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianDto {
    pub id: Uuid,
    pub fullname: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianDetailDto {
    pub id: Uuid,
    pub fullname: String,
    pub phone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CateringDto {
    pub id: Uuid,
    pub name: String,
}
