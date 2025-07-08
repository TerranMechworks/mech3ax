use mech3ax_types::bitflags;

bitflags! {
    pub struct LightFlagsU32: u32 {
        // This flag never occurs in animation definitions, but does in GameZ
        const TRANSLATION_ABS = 1 << 0; // 0x001
        const TRANSLATION = 1 << 1;     // 0x002
        const ROTATION = 1 << 2;        // 0x004
        const RANGE = 1 << 3;           // 0x008
        const COLOR = 1 << 4;           // 0x010
        const AMBIENT = 1 << 5;         // 0x020
        const DIFFUSE = 1 << 6;         // 0x040
        const DIRECTIONAL = 1 << 7;     // 0x080
        const SATURATED = 1 << 8;       // 0x100
        const SUBDIVIDE = 1 << 9;       // 0x200
        const STATIC = 1 << 10;         // 0x400
    }
}

impl LightFlagsU32 {
    pub const DEFAULT: Self = Self::from_bits_truncate(
        Self::SUBDIVIDE.bits()
            | Self::SATURATED.bits()
            | Self::DIRECTIONAL.bits()
            | Self::RANGE.bits()
            | Self::TRANSLATION.bits()
            | Self::TRANSLATION_ABS.bits(),
    );
}

bitflags! {
    pub struct LightFlagsU16: u16 {
        // This flag never occurs in animation definitions, but does in GameZ
        const TRANSLATION_ABS = 1 << 0;
        const TRANSLATION = 1 << 1;
        const ROTATION = 1 << 2;
        const RANGE = 1 << 3;
        const COLOR = 1 << 4;
        const AMBIENT = 1 << 5;
        const DIFFUSE = 1 << 6;
        const DIRECTIONAL = 1 << 7;
        const SATURATED = 1 << 8;
        const SUBDIVIDE = 1 << 9;
        const STATIC = 1 << 10;
    }
}

impl LightFlagsU16 {
    pub const DEFAULT: Self = Self::from_bits_truncate(
        Self::SUBDIVIDE.bits()
            | Self::SATURATED.bits()
            | Self::DIRECTIONAL.bits()
            | Self::RANGE.bits()
            | Self::TRANSLATION.bits()
            | Self::TRANSLATION_ABS.bits(),
    );
}
