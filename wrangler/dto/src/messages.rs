use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use chrono::NaiveTime;

use crate::guardian::GuardianDto;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ContactDto {
    Unknown(String),
    GuardianWithPhone(GuardianDto),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DbMessage {
    pub id: Uuid,
    pub phone: String,
    pub content: String,
    pub sent: Option<NaiveDateTime>,
    pub inserted: NaiveDateTime,
    pub cause_id: Option<Uuid>,
    pub outgoing: bool,
    pub processed: bool,
}

pub fn parse_message(msg: DbMessage) -> Message {
    let metadata = MessageMetadata {
        id: msg.id,
        inserted: msg.inserted,
    };
    let data = MessageData {
        phone: msg.phone,
        content: msg.content,
    };

    match (msg.sent, msg.outgoing) {
        (Some(sent), true) => Message::Sent(SentMessage {
            metadata,
            data,
            sent,
        }),
        (Some(received), false) => Message::Received(ReceivedMessage {
            data,
            metadata,
            received,
            processed: msg.processed,
        }),
        (None, true) => Message::Pending(PendingMessage { data, metadata }),
        _ => panic!("Invalid message combination"),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MessageMetadata {
    pub id: Uuid,
    pub inserted: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MessageData {
    pub phone: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SentMessage {
    pub metadata: MessageMetadata,
    pub data: MessageData,
    pub sent: NaiveDateTime,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ReceivedMessage {
    pub metadata: MessageMetadata,
    pub data: MessageData,
    pub received: NaiveDateTime,
    pub processed: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PendingMessage {
    pub metadata: MessageMetadata,
    pub data: MessageData,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Message {
    Sent(SentMessage),
    Received(ReceivedMessage),
    Pending(PendingMessage),
}

impl Message {
    pub fn metadata(&self) -> &MessageMetadata {
        match self {
            Message::Sent(sent_message) => &sent_message.metadata,
            Message::Received(received_message) => &received_message.metadata,
            Message::Pending(pending_message) => &pending_message.metadata,
        }
    }

    pub fn data(&self) -> &MessageData {
        match self {
            Message::Sent(sent_message) => &sent_message.data,
            Message::Received(received_message) => &received_message.data,
            Message::Pending(pending_message) => &pending_message.data,
        }
    }
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
    NoStudentSpecified,
    UnknownTerm(String),
    AmbiguousTerm(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CancellationResult {
    pub name: String,
    pub meals: HashMap<String, i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageProcessing {
    Init,
    Tokens(Vec<Token>),
    Cancellation(CancellationRequest),
    StudentCancellation(AttendanceCancellation),
    CancellationResult(Vec<CancellationResult>),
    RequestError(RequestError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhoneStatusDto {
    pub last_updated: NaiveDateTime,
    pub total_sent: i32,
    pub total_received: i32,
    pub signal: i32,
}
