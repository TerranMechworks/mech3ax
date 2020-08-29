use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    pub struct NodeBitFlags: u32 {
        // const UNK00 = 1 << 0;
        // const UNK01 = 1 << 1;
        const ACTIVE = 1 << 2;
        const ALTITUDE_SURFACE = 1 << 3;
        const INTERSECT_SURFACE = 1 << 4;
        const INTERSECT_BBOX = 1 << 5;
        // const PROXIMITY = 1 << 6;
        const LANDMARK = 1 << 7;
        const UNK08 = 1 << 8;
        const HAS_MESH = 1 << 9;
        const UNK10 = 1 << 10;
        // const UNK11 = 1 << 11;
        // const UNK12 = 1 << 12;
        // const UNK13 = 1 << 13;
        // const UNK14 = 1 << 14;
        const UNK15 = 1 << 15;
        const CAN_MODIFY = 1 << 16;
        const CLIP_TO = 1 << 17;
        // const UNK18 = 1 << 18;
        const TREE_VALID = 1 << 19;
        // const UNK20 = 1 << 20;
        // const UNK21 = 1 << 21;
        // const UNK22 = 1 << 22;
        // const OVERRIDE = 1 << 23;
        const ID_ZONE_CHECK = 1 << 24;
        const UNK25 = 1 << 25;
        // const UNK26 = 1 << 26;
        // const UNK27 = 1 << 27;
        const UNK28 = 1 << 28;
        // const UNK29 = 1 << 29;
        // const UNK30 = 1 << 30;
        // const UNK31 = 1 << 31;

        const BASE = Self::ACTIVE.bits | Self::TREE_VALID.bits | Self::ID_ZONE_CHECK.bits;
        const DEFAULT = Self::BASE.bits | Self::ALTITUDE_SURFACE.bits | Self::INTERSECT_SURFACE.bits;
    }
}

fn bool_false(value: &bool) -> bool {
    !value
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeFlags {
    #[serde(skip_serializing_if = "bool_false")]
    pub altitude_surface: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub intersect_surface: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub intersect_bbox: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub landmark: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub unk08: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub has_mesh: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub unk10: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub unk15: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub can_modify: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub clip_to: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub unk25: bool,
    #[serde(skip_serializing_if = "bool_false")]
    pub unk28: bool,
}

impl From<NodeBitFlags> for NodeFlags {
    fn from(flags: NodeBitFlags) -> Self {
        Self {
            altitude_surface: flags.contains(NodeBitFlags::ALTITUDE_SURFACE),
            intersect_surface: flags.contains(NodeBitFlags::INTERSECT_SURFACE),
            intersect_bbox: flags.contains(NodeBitFlags::INTERSECT_BBOX),
            landmark: flags.contains(NodeBitFlags::LANDMARK),
            unk08: flags.contains(NodeBitFlags::UNK08),
            has_mesh: flags.contains(NodeBitFlags::HAS_MESH),
            unk10: flags.contains(NodeBitFlags::UNK10),
            unk15: flags.contains(NodeBitFlags::UNK15),
            can_modify: flags.contains(NodeBitFlags::CAN_MODIFY),
            clip_to: flags.contains(NodeBitFlags::CLIP_TO),
            unk25: flags.contains(NodeBitFlags::UNK25),
            unk28: flags.contains(NodeBitFlags::UNK28),
        }
    }
}

impl From<NodeFlags> for NodeBitFlags {
    fn from(flags: NodeFlags) -> Self {
        let mut bitflags = Self::BASE;
        if flags.altitude_surface {
            bitflags |= NodeBitFlags::ALTITUDE_SURFACE;
        }
        if flags.intersect_surface {
            bitflags |= NodeBitFlags::INTERSECT_SURFACE;
        }
        if flags.intersect_bbox {
            bitflags |= NodeBitFlags::INTERSECT_BBOX;
        }
        if flags.landmark {
            bitflags |= NodeBitFlags::LANDMARK;
        }
        if flags.unk08 {
            bitflags |= NodeBitFlags::UNK08;
        }
        if flags.has_mesh {
            bitflags |= NodeBitFlags::HAS_MESH;
        }
        if flags.unk10 {
            bitflags |= NodeBitFlags::UNK10;
        }
        if flags.unk15 {
            bitflags |= NodeBitFlags::UNK15;
        }
        if flags.can_modify {
            bitflags |= NodeBitFlags::CAN_MODIFY;
        }
        if flags.clip_to {
            bitflags |= NodeBitFlags::CLIP_TO;
        }
        if flags.unk25 {
            bitflags |= NodeBitFlags::UNK25;
        }
        if flags.unk28 {
            bitflags |= NodeBitFlags::UNK28;
        }
        bitflags
    }
}
