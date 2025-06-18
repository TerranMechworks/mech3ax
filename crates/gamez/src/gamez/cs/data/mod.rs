mod texture;

use super::HeaderCsC;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Campaign {
    C1,
    C1B,
    C1C,
    C2,
    C2B,
    C3,
    C4,
    C5,
    Unk,
}

impl Campaign {
    pub(crate) fn from_header(header: &HeaderCsC) -> Self {
        if header == &HEADER_C1 {
            Campaign::C1
        } else if header == &HEADER_C1B {
            Campaign::C1B
        } else if header == &HEADER_C1C {
            Campaign::C1C
        } else if header == &HEADER_C2 {
            Campaign::C2
        } else if header == &HEADER_C2B {
            Campaign::C2B
        } else if header == &HEADER_C3 {
            Campaign::C3
        } else if header == &HEADER_C4 {
            Campaign::C4
        } else if header == &HEADER_C5 {
            Campaign::C5
        } else {
            Self::Unk
        }
    }
}

const HEADER_C1: HeaderCsC = HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967277730,
    texture_count: 565,
    textures_offset: 40,
    materials_offset: 24900,
    models_offset: 69060,
    node_array_size: 7064,
    light_index: 2349,
    nodes_offset: 4326296,
};
const HEADER_C1B: HeaderCsC = HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278018,
    texture_count: 359,
    textures_offset: 40,
    materials_offset: 15836,
    models_offset: 60176,
    node_array_size: 5603,
    light_index: 5221,
    nodes_offset: 1924148,
};
const HEADER_C1C: HeaderCsC = HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278208,
    texture_count: 279,
    textures_offset: 40,
    materials_offset: 12316,
    models_offset: 56372,
    node_array_size: 5644,
    light_index: 5262,
    nodes_offset: 1964684,
};
const HEADER_C2: HeaderCsC = HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278462,
    texture_count: 498,
    textures_offset: 40,
    materials_offset: 21952,
    models_offset: 66100,
    node_array_size: 4956,
    light_index: 4956,
    nodes_offset: 3111828,
};
const HEADER_C2B: HeaderCsC = HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278721,
    texture_count: 285,
    textures_offset: 40,
    materials_offset: 12580,
    models_offset: 56728,
    node_array_size: 4901,
    light_index: 4519,
    nodes_offset: 1658700,
};
const HEADER_C3: HeaderCsC = HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278943,
    texture_count: 465,
    textures_offset: 40,
    materials_offset: 20500,
    models_offset: 64648,
    node_array_size: 5408,
    light_index: 5331,
    nodes_offset: 3661748,
};
const HEADER_C4: HeaderCsC = HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967279328,
    texture_count: 654,
    textures_offset: 40,
    materials_offset: 28816,
    models_offset: 72964,
    node_array_size: 8289,
    light_index: 8437,
    nodes_offset: 5107144,
};
const HEADER_C5: HeaderCsC = HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967279700,
    texture_count: 582,
    textures_offset: 40,
    materials_offset: 25648,
    models_offset: 69796,
    node_array_size: 11438,
    light_index: 9979,
    nodes_offset: 5259292,
};
