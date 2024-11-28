use crate::assert_with_msg;
use log::trace;
use mech3ax_types::{u32_to_usize, AsBytes};
use std::io::{Read, Result, Seek, SeekFrom, Write};

#[cfg(not(target_endian = "little"))]
compile_error!("only little-endian architectures are supported");

pub struct CountingReader<R: Read> {
    inner: R,
    pub offset: usize,
    pub prev: usize,
}

impl<R: Read> CountingReader<R> {
    #[inline]
    pub const fn new(read: R) -> Self {
        Self {
            inner: read,
            offset: 0,
            prev: 0,
        }
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    #[inline]
    pub fn into_inner(self) -> R {
        self.inner
    }

    #[inline]
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.inner.read_exact(buf)?;
        self.prev = self.offset;
        self.offset += buf.len();
        Ok(())
    }

    #[inline]
    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    #[inline]
    pub fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    #[inline]
    pub fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    #[inline]
    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    #[inline]
    pub fn read_i16(&mut self) -> Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    pub fn read_struct<S: AsBytes>(&mut self) -> Result<S> {
        let mut s = S::zeroed();
        let buf = s.as_bytes_mut();
        let len = buf.len();
        self.read_exact(buf)?;
        trace!("{:#?} (len: {}, at {})", s, len, self.prev);
        Ok(s)
    }

    pub fn read_string(&mut self) -> crate::Result<String> {
        let offset = self.offset;
        let len = u32_to_usize(self.read_u32()?);
        let mut buf = vec![0u8; len];
        self.read_exact(&mut buf)?;
        trace!("`{}` (len: {}, at {})", buf.escape_ascii(), len, offset);
        if !buf.is_ascii() {
            // is_ascii is optimised, only try and find the invalid character after it
            for (index, b) in (self.prev..).zip(buf.iter()) {
                if b & 0x80 != 0 {
                    return Err(assert_with_msg!("Expected data to be ASCII (at {})", index));
                }
            }
            // technically this should be unreachable
            Err(assert_with_msg!(
                "Expected data to be ASCII (at {})",
                self.offset
            ))
        } else {
            // SAFETY: v is ASCII, and therefore UTF8
            Ok(unsafe { String::from_utf8_unchecked(buf) })
        }
    }

    pub fn assert_end(&mut self) -> crate::Result<()> {
        let mut buf = [0; 1];
        match self.inner.read(&mut buf)? {
            0 => Ok(()),
            _ => Err(assert_with_msg!(
                "Expected all data to be read (at {})",
                self.offset
            )),
        }
    }
}

impl<R: Read + Seek> CountingReader<R> {
    #[inline]
    pub fn seek(&mut self, pos: SeekFrom) -> crate::Result<usize> {
        let offset = self.inner.seek(pos)?;
        let offset = offset
            .try_into()
            .map_err(|_e| assert_with_msg!("File is bigger than 4 GIB"))?;
        self.offset = offset;
        Ok(offset)
    }
}

pub struct CountingWriter<W: Write> {
    inner: W,
    pub offset: usize,
}

impl<W: Write> CountingWriter<W> {
    #[inline]
    pub const fn new(write: W, offset: usize) -> Self {
        Self {
            inner: write,
            offset,
        }
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.inner
    }

    #[inline(always)]
    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.offset += buf.len();
        self.inner.write_all(buf)
    }

    #[inline]
    pub fn write_u32(&mut self, value: u32) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    #[inline]
    pub fn write_i32(&mut self, value: i32) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    #[inline]
    pub fn write_f32(&mut self, value: f32) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    #[inline]
    pub fn write_u16(&mut self, value: u16) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    #[inline]
    pub fn write_i16(&mut self, value: i16) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    #[inline]
    pub fn write_struct<S: AsBytes>(&mut self, value: &S) -> Result<()> {
        let buf = value.as_bytes();
        trace!("{:#?} (len: {}, at {})", value, buf.len(), self.offset);
        self.write_all(buf)
    }

    pub fn write_string(&mut self, value: &str) -> crate::Result<()> {
        if !value.is_ascii() {
            return Err(assert_with_msg!("Expected ASCII string"));
        }
        let buf = value.as_bytes();
        let len: u32 = buf
            .len()
            .try_into()
            .map_err(|_e| assert_with_msg!("String too long"))?;
        trace!(
            "`{}` (len: {}, at {})",
            buf.escape_ascii(),
            len,
            self.offset
        );
        self.write_u32(len)?;
        self.inner.write_all(buf)?;
        Ok(())
    }

    #[inline]
    pub fn write_zeros(&mut self, count: u32) -> Result<()> {
        let buf = vec![0; count as usize];
        self.write_all(&buf)
    }
}

#[cfg(test)]
mod tests;
