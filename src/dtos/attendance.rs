use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetMonthAttendanceDto {
    pub target: Uuid,
    pub year: u32,
    pub month: u32,
}


#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct CateringMealDto{
    pub id: Uuid,
    pub name: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MonthAttendanceDto {
    pub days_of_week: Vec<bool>,
    pub meals: Vec<CateringMealDto>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub attendance: BTreeMap<NaiveDate, BTreeMap<Uuid, u32>>,
}
