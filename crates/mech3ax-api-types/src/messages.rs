use ::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Messages {
    pub language_id: u32,
    pub entries: Vec<(String, u32, String)>,
}
