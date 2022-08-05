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
