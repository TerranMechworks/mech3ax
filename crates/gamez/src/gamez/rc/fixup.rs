use super::HeaderRcC;
use crate::gamez::common::{SIGNATURE, VERSION_RC};
use log::debug;

macro_rules! header_m6 {
    ($node_count:literal) => {
        HeaderRcC {
            signature: SIGNATURE,
            version: VERSION_RC,
            texture_count: 612,
            textures_offset: 36,
            materials_offset: 22068,
            models_offset: 242084,
            node_array_size: 16000,
            node_count: $node_count,
            nodes_offset: 2299168,
        }
    };
}

const HEADER_M6_READ: HeaderRcC = header_m6!(309);
const HEADER_M6_WRITE: HeaderRcC = header_m6!(4955);

pub(super) fn read(header: &mut HeaderRcC) {
    if header == &HEADER_M6_READ {
        debug!(
            "m6 header node_count fixup: `{}` -> `{}`",
            header.node_count, HEADER_M6_WRITE.node_count
        );
        header.node_count = HEADER_M6_WRITE.node_count;
    }
}

pub(super) fn write(header: &mut HeaderRcC) {
    if header == &HEADER_M6_WRITE {
        debug!(
            "m6 header node_count fixup: `{}` <- `{}`",
            HEADER_M6_READ.node_count, header.node_count
        );
        header.node_count = HEADER_M6_READ.node_count;
    }
}
