pub trait ReprSize {
    const SIZE: u32;
    fn _static_assert_size();
}

#[macro_export]
macro_rules! static_assert_size {
    ($type:ty, $size:expr) => {
        #[allow(dead_code)]
        impl $crate::ReprSize for $type {
            const SIZE: u32 = $size;

            fn _static_assert_size() {
                const _: [(); $size as usize] = [(); ::std::mem::size_of::<$type>()];
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
