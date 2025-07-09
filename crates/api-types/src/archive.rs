//! Archive-based `*.zbd` data structures.
use crate::serde::bytes;
use crate::{api, sum};
use mech3ax_timestamp::DateTime;

api! {
    struct ArchiveEntry {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        rename: Option<String> = { None },
        flags: u32,
        info: ArchiveEntryInfo,
    }
}

sum! {
    enum ArchiveEntryInfo {
        Valid(ArchiveEntryInfoValid),
        Invalid(ArchiveEntryInfoInvalid),
    }
}

api! {
    struct ArchiveEntryInfoValid {
        comment: String,
        datetime: DateTime,
    }
}

api! {
    struct ArchiveEntryInfoInvalid {
        #[serde(with = "bytes")]
        comment: Vec<u8>,
        filetime: u64,
    }
}
