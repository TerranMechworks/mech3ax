pub trait ConstSize {
    const SIZE: usize;
    fn _static_assert_size();
}

// annoyingly, the one in mech3ax_common is u32, not usize
macro_rules! static_assert_size {
    ($type:ty, $size:expr) => {
        #[allow(dead_code)]
        impl $crate::size::ConstSize for $type {
            const SIZE: usize = $size;

            fn _static_assert_size() {
                const _: [(); $size] = [(); ::std::mem::size_of::<$type>()];
            }
        }
    };
}

pub(crate) use static_assert_size;
