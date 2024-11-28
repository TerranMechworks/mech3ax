use bytemuck::{AnyBitPattern, NoUninit};

#[cfg(not(target_endian = "little"))]
compile_error!("only little-endian architectures are supported");

/// A trait that ensures a structure has a known size (in bytes), and can be
/// read as bytes.
///
/// Do not implement this manually, instead use [`impl_as_bytes!`].
pub trait AsBytes: NoUninit + AnyBitPattern + std::fmt::Debug {
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
