use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageEntry {
    pub key: String,
    pub id: u32,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Messages {
    pub language_id: u32,
    pub entries: Vec<MessageEntry>,
}
