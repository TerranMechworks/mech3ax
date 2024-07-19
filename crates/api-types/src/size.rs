use bytemuck::{AnyBitPattern, NoUninit};

/// A trait that ensures a structure has a known size (in bytes), and can be
/// read from and written to disk (as bytes).
///
/// This is rather involved.
pub trait ReprSize: NoUninit + AnyBitPattern + Sized + 'static {
    /// The size of the structure in bytes.
    const SIZE: u32;

    /// A compile-time assertion that the size of the structure is correct.
    const _ASSERT_SIZE: ();

    /// Borrow the structure's memory as bytes.
    fn as_bytes(&self) -> &[u8];

    /// Borrow the structure's memory as bytes.
    fn as_bytes_mut(&mut self) -> &mut [u8];
}

#[macro_export]
macro_rules! static_assert_size {
    ($type:ty, $size:literal) => {
        impl $crate::ReprSize for $type {
            #[allow(dead_code)]
            const SIZE: u32 = $size;

            #[allow(dead_code)]
            const _ASSERT_SIZE: () = {
                const _: [(); $size] = [(); ::std::mem::size_of::<$type>()];
            };

            fn as_bytes(&self) -> &[u8] {
                let b: &[u8; $size] = ::bytemuck::must_cast_ref(self);
                b
            }

            fn as_bytes_mut(&mut self) -> &mut [u8] {
                let b: &mut [u8; $size] = ::bytemuck::must_cast_mut(self);
                b
            }
        }
    };
}

// Required for cast safety, should be valid on 32 bit platforms and up.
// The assumption here is that with these properties, and the fact that for
// integers all bit patterns should be valid, casting should be safe. However,
// there may be some weirdo platforms where this doesn't hold.
const _: () = assert!(std::mem::size_of::<usize>() >= std::mem::size_of::<u32>());
const _: () = assert!(std::mem::size_of::<usize>() >= std::mem::size_of::<u16>());
const _: () = assert!((usize::MAX as u128) >= (u32::MAX as u128));
const _: () = assert!((usize::MAX as u128) >= (u16::MAX as u128));
const _: () = assert!((usize::MIN as u128) == (u32::MIN as u128));
const _: () = assert!((usize::MIN as u128) == (u16::MIN as u128));

#[inline(always)]
pub fn u32_to_usize(value: u32) -> usize {
    // Cast safety: guarded by assert above
    value as _
}

#[inline(always)]
pub fn u16_to_usize(value: u16) -> usize {
    // Cast safety: guarded by assert above
    value as _
}
