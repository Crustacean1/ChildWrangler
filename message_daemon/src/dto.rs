use std::collections::HashMap;

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Student {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
    pub grace_period: NaiveTime,
    pub meals: Vec<Meal>,
    pub starts: NaiveDate,
    pub ends: NaiveDate,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Meal {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageProcessingContext {
    pub guardian_id: Uuid,
    pub fullname: String,
    pub students: Vec<Student>,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Token {
    Student(Uuid),
    Date(NaiveDate),
    Meal(Uuid),
    Unknown(String),
    Ambiguous(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancellationRequest {
    pub since: NaiveDate,
    pub until: NaiveDate,
    pub students: Vec<Uuid>,
    pub meals: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StudentCancellation {
    pub id: Uuid,
    pub meals: Vec<Uuid>,
    pub since: NaiveDate,
    pub until: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttendanceCancellation {
    pub students: Vec<StudentCancellation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RequestError {
    InvalidTimeRange,
    TooManyDates,
    NoDateSpecified,
    UnknownTerm,
    AmbiguousTerm,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancellationResult {
    pub name: String,
    pub meals: HashMap<String, i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageProcessing {
    Context(Vec<Student>),
    Tokens(Vec<Token>),
    Cancellation(CancellationRequest),
    StudentCancellation(Vec<StudentCancellation>),
    RequestError(RequestError),
}
