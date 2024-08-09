use crate::size::FromBytes;
use mech3ax_common::PeError as Error;

#[cfg(not(target_endian = "little"))]
compile_error!("only little-endian architectures are supported");

type Result<T> = ::std::result::Result<T, Error>;

pub trait StructAt {
    fn struct_at<S: FromBytes>(&self, offset: usize) -> Result<S>;
}

impl StructAt for &[u8] {
    fn struct_at<S: FromBytes>(&self, offset: usize) -> Result<S> {
        let size = std::mem::size_of::<S>();
        let end = offset
            .checked_add(size)
            .ok_or(Error::ReadOutOfBounds(offset))?;
        let bytes = self.get(offset..end).ok_or(Error::ReadOutOfBounds(end))?;
        Ok(bytemuck::pod_read_unaligned(bytes))
    }
}
