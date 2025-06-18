mod c1;
mod c1b;
mod c1c;
mod c2;
mod c2b;
mod c3;
mod c4;
mod c5;

use super::Campaign;

impl Campaign {
    pub(crate) fn image_ptrs(self) -> &'static [u32] {
        match self {
            Self::C1 => c1::IMAGE_PTR,
            Self::C1B => c1b::IMAGE_PTR,
            Self::C1C => c1c::IMAGE_PTR,
            Self::C2 => c2::IMAGE_PTR,
            Self::C2B => c2b::IMAGE_PTR,
            Self::C3 => c3::IMAGE_PTR,
            Self::C4 => c4::IMAGE_PTR,
            Self::C5 => c5::IMAGE_PTR,
            Self::Unk => &[],
        }
    }
}
