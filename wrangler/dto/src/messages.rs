use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use chrono::NaiveTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianDto {
    pub id: Uuid,
    pub fullname: String,
    pub phone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ContactDto {
    Unknown(String),
    GuardianWithPhone(GuardianDto),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianStudent {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub id: i32,
    pub sent: NaiveDateTime,
    pub content: String,
    pub msg_type: MessageType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MessageType {
    Sent,
    Received(bool),
    Pending,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianDetails {
    pub id: Uuid,
    pub fullname: String,
    pub phone: Option<String>,
    pub students: Vec<GuardianStudent>,
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhoneStatusDto {
    pub last_updated: NaiveDateTime,
    pub total_sent: i32,
    pub total_received: i32,
    pub signal: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageState {
    Received,
    Outgoing,
    Sent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GeneralMessageDto {
    pub msg_type: MessageState,
    pub message_id: i32,
    pub sent: NaiveDateTime,
    pub received: NaiveDateTime,
    pub sender_id: Option<Uuid>,
    pub sender: String,
    pub content: String,
}
