//! Localisation data structures.
use crate::api;

api! {
    struct MessageEntry {
        key: String,
        id: u32,
        value: String,
    }
}

api! {
    struct Messages {
        language_id: u32,
        entries: Vec<MessageEntry>,
    }
}
