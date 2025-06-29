//! Interpreter (`interp.zbd`) data structures.
use crate::fld;
use mech3ax_timestamp::DateTime;

fld! {
    struct Script {
        name: String,
        datetime: DateTime,
        lines: Vec<String>,
    }
}
