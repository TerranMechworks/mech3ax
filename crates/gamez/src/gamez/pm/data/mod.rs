mod texture;

use super::HeaderPmC;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Campaign {
    C1,
    C2,
    C3,
    C4,
    Unk,
}

impl Campaign {
    pub(crate) fn from_header(header: &HeaderPmC) -> Self {
        // could probably just use the timestamp...
        if header == &HEADER_C1 {
            Self::C1
        } else if header == &HEADER_C2 {
            Self::C2
        } else if header == &HEADER_C3 {
            Self::C3
        } else if header == &HEADER_C4 {
            Self::C4
        } else {
            Self::Unk
        }
    }
}

const HEADER_C1: HeaderPmC = HeaderPmC {
    signature: 43455010,
    version: 41,
    timestamp: 942371914,
    texture_count: 437,
    textures_offset: 40,
    materials_offset: 19268,
    models_offset: 65452,
    node_array_size: 5793,
    node_count: 1240,
    nodes_offset: 4909452,
};

const HEADER_C2: HeaderPmC = HeaderPmC {
    signature: 43455010,
    version: 41,
    timestamp: 942426872,
    texture_count: 396,
    textures_offset: 40,
    materials_offset: 17464,
    models_offset: 63648,
    node_array_size: 5593,
    node_count: 835,
    nodes_offset: 3609596,
};

const HEADER_C3: HeaderPmC = HeaderPmC {
    signature: 43455010,
    version: 41,
    timestamp: 942371490,
    texture_count: 362,
    textures_offset: 40,
    materials_offset: 15968,
    models_offset: 62152,
    node_array_size: 3541,
    node_count: 3541,
    nodes_offset: 2992964,
};

const HEADER_C4: HeaderPmC = HeaderPmC {
    signature: 43455010,
    version: 41,
    timestamp: 942371540,
    texture_count: 400,
    textures_offset: 40,
    materials_offset: 17640,
    models_offset: 63824,
    node_array_size: 5755,
    node_count: 1822,
    nodes_offset: 3833584,
};
