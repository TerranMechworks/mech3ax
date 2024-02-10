bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LightFlags: u32 {
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

        const DEFAULT = Self::SUBDIVIDE.bits()
        | Self::SATURATED.bits()
        | Self::DIRECTIONAL.bits()
        | Self::RANGE.bits()
        | Self::TRANSLATION.bits()
        | Self::TRANSLATION_ABS.bits();
    }
}
