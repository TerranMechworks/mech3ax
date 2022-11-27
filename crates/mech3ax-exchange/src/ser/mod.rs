mod io_writer;

use crate::error::Result;

/// Serialize a value to binary zlisp data.
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + serde::Serialize,
{
    let writer = std::io::Cursor::new(Vec::new());
    let mut serializer = io_writer::IoWriter::new(writer);
    value.serialize(&mut serializer)?;
    let cursor = serializer.finish()?;
    Ok(cursor.into_inner())
}

/// Serialize a value to binary zlisp data.
#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    T: ?Sized + serde::Serialize,
    W: std::io::Write,
{
    let mut serializer = io_writer::IoWriter::new(writer);
    value.serialize(&mut serializer)?;
    let _ = serializer.finish()?;
    Ok(())
}
