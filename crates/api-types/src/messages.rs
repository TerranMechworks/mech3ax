//! Localisation data structures.
use crate::fld;

fld! {
    struct MessageEntry {
        key: String,
        id: u32,
        value: String,
    }
}

fld! {
    struct Messages {
        language_id: u32,
        entries: Vec<MessageEntry>,
    }
}
