//! Archive-based `*.zbd` data structures.
use crate::serde::bytes;
use crate::{fld, sum};
use mech3ax_timestamp::DateTime;

fld! {
    struct ArchiveEntry {
        name: String,
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

fld! {
    struct ArchiveEntryInfoValid {
        comment: String,
        datetime: DateTime,
    }
}

fld! {
    struct ArchiveEntryInfoInvalid {
        #[serde(with = "bytes")]
        comment: Vec<u8>,
        filetime: u64,
    }
}
