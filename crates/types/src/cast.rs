// Required for cast safety, should be valid on 32 bit platforms and up.
// The assumption here is that with these properties, and the fact that for
// integers all bit patterns should be valid, casting should be safe. However,
// there may be some weirdo platforms where this doesn't hold.
const _: () = assert!(std::mem::size_of::<usize>() >= std::mem::size_of::<u16>());
const _: () = assert!(std::mem::size_of::<usize>() >= std::mem::size_of::<u32>());

const _: () = assert!((usize::MIN as u128) == (u16::MIN as u128));
const _: () = assert!((usize::MIN as u128) == (u32::MIN as u128));

const _: () = assert!((usize::MAX as u128) >= (u16::MAX as u128));
const _: () = assert!((usize::MAX as u128) >= (u32::MAX as u128));

const _: () = assert!((i64::MIN as i128) < (u32::MIN as i128));
const _: () = assert!((i64::MAX as i128) > (u32::MAX as i128));

#[inline(always)]
pub const fn u16_to_usize(value: u16) -> usize {
    // Cast safety: guarded by assert above
    value as _
}

#[inline(always)]
pub const fn u32_to_usize(value: u32) -> usize {
    // Cast safety: guarded by assert above
    value as _
}

#[inline(always)]
pub const fn u32_to_i64(value: u32) -> i64 {
    // Cast safety: guarded by assert above
    value as _
}

pub trait AsUsize: Sized + Copy + Send + Sync + 'static {
    fn as_usize(self) -> usize;
}

impl AsUsize for u16 {
    #[inline]
    fn as_usize(self) -> usize {
        u16_to_usize(self)
    }
}

impl AsUsize for u32 {
    #[inline]
    fn as_usize(self) -> usize {
        u32_to_usize(self)
    }
}
