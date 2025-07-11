use crate::{api, num, sum, Color, IndexR};

api! {
    struct CycleData {
        texture_indices: Vec<IndexR>,
        looping: bool,
        speed: f32,
        current_frame: i32,
        cycle_ptr: u32,
        tex_map_ptr: u32,
    }
}

num! {
    enum Soil: u32 {
        Default = 0,
        Water = 1,
        Seafloor = 2,
        Quicksand = 3,
        Lava = 4,
        Fire = 5,
        Dirt = 6,
        Mud = 7,
        Grass = 8,
        Concrete = 9,
        Snow = 10,
        Mech = 11,
        Silt = 12,
        NoSlip = 13,
    }
}

impl Soil {
    #[inline]
    pub const fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl Default for Soil {
    #[inline]
    fn default() -> Self {
        Self::Default
    }
}

api! {
    struct TexturedMaterial {
        texture_index: IndexR,
        soil: Soil = { Soil::Default },
        cycle: Option<CycleData> = { None },
        flag: bool,
    }
}

api! {
    struct ColoredMaterial {
        color: Color,
        alpha: u8,
        soil: Soil = { Soil::Default },
    }
}

sum! {
    enum Material {
        Textured(TexturedMaterial),
        Colored(ColoredMaterial),
    }
}

impl Material {
    #[inline]
    pub fn is_cycled(&self) -> bool {
        match self {
            Self::Colored(_) => false,
            Self::Textured(textured) => textured.cycle.is_some(),
        }
    }
}
