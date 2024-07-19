// mod slice_reader;
mod io_reader;

use crate::error::Result;

#[inline]
pub fn from_slice<'a, T>(slice: &'a [u8]) -> Result<T>
where
    T: serde::Deserialize<'a>,
{
    let mut reader = io_reader::IoReader::new(slice);
    T::deserialize(&mut reader)
}

#[inline]
pub fn from_reader<R, T>(reader: R) -> Result<T>
where
    T: serde::de::DeserializeOwned,
    R: std::io::Read,
{
    let mut reader = io_reader::IoReader::new(reader);
    T::deserialize(&mut reader)
}
