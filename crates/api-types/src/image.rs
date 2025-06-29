//! Image/texture data structures.
use crate::serde::bytes;
use crate::{api, num, sum};

num! {
    enum TextureAlpha {
        None = 0,
        Simple = 1,
        Full = 2,
    }
}

num! {
    enum TextureStretch: u16 {
        None = 0,
        Vertical = 1,
        Horizontal = 2,
        Both = 3,
        /// Crimson Skies only
        Unk4 = 4,
        /// Crimson Skies only
        Unk7 = 7,
        /// Crimson Skies only
        Unk8 = 8,
    }
}

api! {
    struct PaletteData {
        #[serde(with = "bytes")]
        data: Vec<u8>,
    }
}

api! {
    struct GlobalPalette {
        index: u32,
        count: u16,
    }
}

sum! {
    enum TexturePalette {
        None,
        Local(PaletteData),
        Global(GlobalPalette),
    }
}

api! {
    struct TextureInfo {
        name: String,
        rename: Option<String> = { None },
        alpha: TextureAlpha,
        width: u16,
        height: u16,
        stretch: TextureStretch,
        image_loaded: bool,
        alpha_loaded: bool,
        palette_loaded: bool,
        palette: TexturePalette,
    }
}

api! {
    struct TextureManifest {
        texture_infos: Vec<TextureInfo>,
        global_palettes: Vec<PaletteData>,
    }
}
