use std::collections::{BTreeMap, HashMap};

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetMonthAttendanceDto {
    pub target: Uuid,
    pub year: u32,
    pub month: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CateringMealDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MonthAttendanceDto {
    pub days_of_week: Vec<bool>,
    pub meals: Vec<CateringMealDto>,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub attendance: BTreeMap<NaiveDate, BTreeMap<Uuid, u32>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Copy)]
pub enum EffectiveAttendance {
    Present,
    Cancelled,
    Absent,
    Blocked,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EffectiveMonthAttendance {
    pub is_student: bool,
    pub attendance: BTreeMap<NaiveDate, BTreeMap<Uuid, EffectiveAttendance>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetEffectiveMonthAttendance {
    pub target: Uuid,
    pub year: i32,
    pub month: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateAttendanceDto {
    pub target: Uuid,
    pub days: Vec<NaiveDate>,
    #[serde(default)]
    pub active_meals: Vec<Uuid>,
    #[serde(default)]
    pub inactive_meals: Vec<Uuid>,
    pub note: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetAttendanceHistoryDto {
    pub date: NaiveDate,
    pub target: Uuid,
    pub meal_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AttendanceItemDto {
    Cancellation(i32, String, String),
    Override(String, bool),
    Init,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttendanceHistoryItemDto {
    pub time: NaiveDateTime,
    pub item: AttendanceItemDto,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MealStatus {
    Init,
    Overriden,
    Cancelled,
    Blocked(Uuid),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttendanceHistoryDto {
    pub status: MealStatus,
    pub events: Vec<AttendanceHistoryItemDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetAttendanceBreakdownDto {
    pub meal_id: Uuid,
    pub target: Uuid,
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttendanceBreakdownDto {
    pub meal: String,
    pub attendance: Vec<(String, (Uuid, i64,i64))>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MonthlyStudentAttendanceDto {
    pub student_id: Uuid,
    pub name: String,
    pub surname: String,
    pub attendance: u32,
    pub group: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, PartialEq)]
pub enum AttendanceOverviewType {
    Present,
    Cancelled,
    Disabled,
    Allergic(Vec<(Uuid, String)>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AttendanceOverviewDto {
    pub student_list: HashMap<Uuid, Vec<(Uuid, String, String, bool)>>,
    pub meal_list: Vec<(Uuid, String)>,
    pub attendance: HashMap<Uuid, HashMap<AttendanceOverviewType, i64>>,
}
