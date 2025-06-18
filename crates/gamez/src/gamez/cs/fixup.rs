use super::HeaderCsC;
use crate::gamez::common::{SIGNATURE, VERSION_CS};

/*
C1A
HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967277730,
    texture_count: 565,
    textures_offset: 40,
    materials_offset: 24900,
    models_offset: 69060,
    node_array_size: 7064,
    node_count: 2349,
    nodes_offset: 4326296,
}
C1B
HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278018,
    texture_count: 359,
    textures_offset: 40,
    materials_offset: 15836,
    models_offset: 60176,
    node_array_size: 5603,
    node_count: 5221,
    nodes_offset: 1924148,
}
C1C
HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278208,
    texture_count: 279,
    textures_offset: 40,
    materials_offset: 12316,
    models_offset: 56372,
    node_array_size: 5644,
    node_count: 5262,
    nodes_offset: 1964684,
}
C2A
HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278462,
    texture_count: 498,
    textures_offset: 40,
    materials_offset: 21952,
    models_offset: 66100,
    node_array_size: 4956,
    node_count: 4956,
    nodes_offset: 3111828,
}
C2B
HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278721,
    texture_count: 285,
    textures_offset: 40,
    materials_offset: 12580,
    models_offset: 56728,
    node_array_size: 4901,
    node_count: 4519,
    nodes_offset: 1658700,
}
C3
HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967278943,
    texture_count: 465,
    textures_offset: 40,
    materials_offset: 20500,
    models_offset: 64648,
    node_array_size: 5408,
    node_count: 5331,
    nodes_offset: 3661748,
}
C5
HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967279700,
    texture_count: 582,
    textures_offset: 40,
    materials_offset: 25648,
    models_offset: 69796,
    node_array_size: 11438,
    node_count: 9979,
    nodes_offset: 5259292,
}
C4
HeaderCsC {
    signature: 43455010,
    version: 42,
    timestamp: 967279328,
    texture_count: 654,
    textures_offset: 40,
    materials_offset: 28816,
    models_offset: 72964,
    node_array_size: 8289,
    node_count: 8437,
    nodes_offset: 5107144,
}
Planes
HeaderCsC {
    timestamp: 967277477,
    texture_count: 298,
    textures_offset: 40,
    materials_offset: 13152,
    models_offset: 103388,
    node_array_size: 3317,
    node_count: 2338,
    nodes_offset: 4881228,
}
*/

const HEADER_C4: HeaderCsC = HeaderCsC {
    signature: SIGNATURE,
    version: VERSION_CS,
    timestamp: 967279328,
    texture_count: 654,
    textures_offset: 40,
    materials_offset: 28816,
    models_offset: 72964,
    node_array_size: 8289,
    light_index: 8437,
    nodes_offset: 5107144,
};

const HEADER_PLANES: HeaderCsC = HeaderCsC {
    signature: SIGNATURE,
    version: VERSION_CS,
    timestamp: 967277477,
    texture_count: 298,
    textures_offset: 40,
    materials_offset: 13152,
    models_offset: 103388,
    node_array_size: 3317,
    light_index: 2338,
    nodes_offset: 4881228,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fixup {
    None,
    C4,
    Planes,
}

impl Fixup {
    pub(super) fn read(header: &HeaderCsC) -> Self {
        if header == &HEADER_C4 {
            Self::C4
        } else if header == &HEADER_PLANES {
            Self::Planes
        } else {
            Self::None
        }
    }

    pub(super) fn write(header: &HeaderCsC) -> Self {
        if header == &HEADER_C4 {
            Self::C4
        } else if header == &HEADER_PLANES {
            Self::Planes
        } else {
            Self::None
        }
    }

    pub(crate) fn model_index_remap(&self, expected_index: i32) -> i32 {
        match self {
            Self::None => expected_index,
            Self::Planes => match expected_index {
                1396 => 1321,
                1398 => 1316,
                1554 => 1779,
                1558 => 1396,
                1559 => 1553,
                1681 => 1502,
                1317 => 1558,
                1460 => 1680,
                1322 => 1557,
                1503 => 1391,
                1314 => 1459,
                1392 => 1395,
                other => other,
            },
            Self::C4 => match expected_index {
                2308 => 2268,
                2371 => 2285,
                2283 => 2281,
                2290 => 2307,
                2367 => 2359,
                2266 => 2368,
                2289 => 2286,
                2288 => 2409,
                2282 => 2366,
                2360 => 2296,
                2375 => 2370,
                2403 => 2309,
                2278 => 2304,
                2408 => 2402,
                2409 => 2403,
                2270 => 2288,
                2300 => 2270,
                2305 => 2276,
                2306 => 2404,
                2285 => 2290,
                2302 => 2273,
                2279 => 2282,
                2292 => 2294,
                2311 => 2272,
                2304 => 2275,
                2293 => 2291,
                2276 => 2277,
                2296 => 2293,
                2314 => 2311,
                2405 => 2292,
                2309 => 2406,
                2307 => 2300,
                2369 => 2405,
                2277 => 2280,
                2299 => 2306,
                2271 => 2301,
                2281 => 2278,
                2297 => 2374,
                2312 => 2490,
                2404 => 2407,
                2269 => 2308,
                2265 => 2303,
                2273 => 2313,
                2291 => 2289,
                2286 => 2269,
                2294 => 2263,
                2301 => 2302,
                2272 => 2310,
                2303 => 2299,
                2310 => 2305,
                2406 => 2408,
                2407 => 2265,
                2274 => 2271,
                2410 => 2297,
                other => other,
            },
        }
    }

    pub(crate) fn last_index_remap(&self, last_index: i32) -> i32 {
        match self {
            Self::None => last_index,
            Self::Planes => {
                if last_index == 1779 {
                    1313
                } else {
                    last_index
                }
            }
            Self::C4 => {
                if last_index == 2490 {
                    2283
                } else {
                    last_index
                }
            }
        }
    }
}
