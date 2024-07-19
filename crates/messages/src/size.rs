use bytemuck::{AnyBitPattern, NoUninit};

/// A trait that ensures a structure has a known size (in bytes), and can be
/// read memory.
///
/// This is rather involved.
pub trait ConstSize: NoUninit + AnyBitPattern + Sized + 'static {
    /// The size of the structure in bytes.
    const SIZE: usize;

    /// A compile-time assertion that the size of the structure is correct.
    const _ASSERT_SIZE: ();
}

// annoyingly, the one in mech3ax_common is u32, not usize
macro_rules! static_assert_size {
    ($type:ty, $size:expr) => {
        impl $crate::size::ConstSize for $type {
            #[allow(dead_code)]
            const SIZE: usize = $size;

            #[allow(dead_code)]
            const _ASSERT_SIZE: () = {
                const _: [(); $size] = [(); ::std::mem::size_of::<$type>()];
            };
        }
    };
}
pub(crate) use static_assert_size;

pub use mech3ax_api_types::{u16_to_usize, u32_to_usize};
