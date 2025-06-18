mod c1;
mod c2;
mod c3;
mod c4;

use super::Campaign;

impl Campaign {
    pub(crate) fn image_ptrs(self) -> &'static [u32] {
        match self {
            Self::C1 => c1::IMAGE_PTR,
            Self::C2 => c2::IMAGE_PTR,
            Self::C3 => c3::IMAGE_PTR,
            Self::C4 => c4::IMAGE_PTR,
            Self::Unk => &[],
        }
    }
}
