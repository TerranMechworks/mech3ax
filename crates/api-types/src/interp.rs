//! Interpreter (`interp.zbd`) data structures.
use crate::api;
use mech3ax_timestamp::DateTime;

api! {
    struct Script {
        name: String,
        datetime: DateTime,
        lines: Vec<String>,
    }
}
