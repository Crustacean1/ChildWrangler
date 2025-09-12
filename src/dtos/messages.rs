use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub msg_type: MessageType
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Sent,
    Received(bool)
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuardianDetails {
    pub id: Uuid,
    pub fullname: String,
    pub phone: Option<String>,
    pub students: Vec<GuardianStudent>,
    pub messages: Vec<Message>,
}
