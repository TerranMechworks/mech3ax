use mech3ax_api_types::nodes::NodeFlags;
use mech3ax_types::bitflags;

bitflags! {
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
        // const UNK12 = 1 << 12; // CS only
        // const UNK13 = 1 << 13;
        // const UNK14 = 1 << 14;
        const TERRAIN = 1 << 15;
        const CAN_MODIFY = 1 << 16; // CS never
        const CLIP_TO = 1 << 17; // CS never
        // const UNK18 = 1 << 18;
        const TREE_VALID = 1 << 19;
        // const UNK20 = 1 << 20;
        // const UNK21 = 1 << 21;
        // const UNK22 = 1 << 22;
        // const OVERRIDE = 1 << 23; // CS only
        const ID_ZONE_CHECK = 1 << 24;
        const UNK25 = 1 << 25;
        // const UNK26 = 1 << 26;
        // const UNK27 = 1 << 27;
        const UNK28 = 1 << 28;
        // const UNK29 = 1 << 29;
        // const UNK30 = 1 << 30;
        // const UNK31 = 1 << 31;
    }
}

impl NodeBitFlags {
    pub(crate) const BASE: Self = Self::from_bits_truncate(
        Self::ACTIVE.bits() | Self::TREE_VALID.bits() | Self::ID_ZONE_CHECK.bits(),
    );
    pub(crate) const DEFAULT: Self = Self::from_bits_truncate(
        Self::BASE.bits() | Self::ALTITUDE_SURFACE.bits() | Self::INTERSECT_SURFACE.bits(),
    );

    #[inline]
    pub(crate) const fn mask_not(self, v: Self) -> Self {
        Self(self.0 & (!v.0))
    }
}

impl From<NodeBitFlags> for NodeFlags {
    fn from(flags: NodeBitFlags) -> Self {
        Self {
            active: flags.contains(NodeBitFlags::ACTIVE),
            altitude_surface: flags.contains(NodeBitFlags::ALTITUDE_SURFACE),
            intersect_surface: flags.contains(NodeBitFlags::INTERSECT_SURFACE),
            intersect_bbox: flags.contains(NodeBitFlags::INTERSECT_BBOX),
            landmark: flags.contains(NodeBitFlags::LANDMARK),
            unk08: flags.contains(NodeBitFlags::UNK08),
            has_mesh: flags.contains(NodeBitFlags::HAS_MESH),
            unk10: flags.contains(NodeBitFlags::UNK10),
            terrain: flags.contains(NodeBitFlags::TERRAIN),
            can_modify: flags.contains(NodeBitFlags::CAN_MODIFY),
            clip_to: flags.contains(NodeBitFlags::CLIP_TO),
            tree_valid: flags.contains(NodeBitFlags::TREE_VALID),
            id_zone_check: flags.contains(NodeBitFlags::ID_ZONE_CHECK),
            unk25: flags.contains(NodeBitFlags::UNK25),
            unk28: flags.contains(NodeBitFlags::UNK28),
        }
    }
}

impl From<&NodeFlags> for NodeBitFlags {
    fn from(flags: &NodeFlags) -> Self {
        let mut bitflags = Self::empty();
        if flags.active {
            bitflags |= NodeBitFlags::ACTIVE;
        }
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
        if flags.terrain {
            bitflags |= NodeBitFlags::TERRAIN;
        }
        if flags.can_modify {
            bitflags |= NodeBitFlags::CAN_MODIFY;
        }
        if flags.clip_to {
            bitflags |= NodeBitFlags::CLIP_TO;
        }
        if flags.tree_valid {
            bitflags |= NodeBitFlags::TREE_VALID;
        }
        if flags.id_zone_check {
            bitflags |= NodeBitFlags::ID_ZONE_CHECK;
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

bitflags! {
    pub struct NodeBitFlagsCs: u32 {
        const UNK02 = 1 << 2;
        const UNK03 = 1 << 3;
        const UNK04 = 1 << 4;
        const UNK05 = 1 << 5;
        const UNK07 = 1 << 7;
        const UNK08 = 1 << 8;
        const UNK09 = 1 << 9;
        const UNK10 = 1 << 10;
        const UNK12 = 1 << 12;
        const UNK15 = 1 << 15;
        const UNK19 = 1 << 19;
        const UNK23 = 1 << 23;
        const UNK24 = 1 << 24;
        const UNK25 = 1 << 25;
    }
}

impl NodeBitFlagsCs {
    #[inline]
    pub(crate) const fn mask_not(self, v: Self) -> Self {
        Self(self.0 & (!v.0))
    }

    #[inline]
    pub(crate) const fn mask(self, v: Self) -> Self {
        Self(self.0 & v.0)
    }
}
