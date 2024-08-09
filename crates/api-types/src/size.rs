use bytemuck::{AnyBitPattern, NoUninit};

#[cfg(not(target_endian = "little"))]
compile_error!("only little-endian architectures are supported");

/// A trait that ensures a structure has a known size (in bytes), and can be
/// read as bytes.
///
/// Do not implement this manually, instead use [`impl_as_bytes!`].
pub trait AsBytes: NoUninit + AnyBitPattern {
    /// The size of the structure in bytes.
    const SIZE: u32;

    /// A compile-time assertion that the size of the structure is correct.
    ///
    /// Although the bytemuck functions also validate the size, in my
    /// experience this check catches errors quicker and more obviously.
    const _ASSERT_SIZE: ();

    /// Borrow the structure's memory as bytes.
    ///
    /// Must not/does not panic at runtime.
    fn as_bytes(&self) -> &[u8];

    /// Borrow the structure's memory as bytes.
    ///
    /// Must not/does not panic at runtime.
    fn as_bytes_mut(&mut self) -> &mut [u8];
}

#[macro_export]
macro_rules! impl_as_bytes {
    ($type:ty, $size:literal) => {
        impl $crate::AsBytes for $type {
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
