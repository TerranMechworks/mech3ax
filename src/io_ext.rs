use crate::assert::{assert_utf8, AssertionError};
use crate::string::str_from_c_sized;
use std::io::{Read, Result, Write};
use std::mem::MaybeUninit;

pub trait FromUsize {
    fn from_usize(value: usize) -> Self;
}

impl FromUsize for usize {
    fn from_usize(value: usize) -> Self {
        value
    }
}

pub trait ReadHelper {
    fn read_u32(&mut self) -> Result<u32>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_f32(&mut self) -> Result<f32>;
    fn read_u16(&mut self) -> Result<u16>;
    fn read_i16(&mut self) -> Result<i16>;
    fn read_struct<S>(&mut self) -> Result<S>;
    fn assert_end(&mut self) -> crate::Result<()>;
    fn read_string<T>(&mut self, offset: &mut T) -> crate::Result<String>
    where
        T: std::ops::AddAssign + FromUsize + std::fmt::Display + Copy;
}

impl<R> ReadHelper for R
where
    R: Read,
{
    fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_f32(&mut self) -> Result<f32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_i16(&mut self) -> Result<i16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    fn read_struct<S>(&mut self) -> Result<S> {
        let size = std::mem::size_of::<S>();
        unsafe {
            let mut mem = MaybeUninit::uninit().assume_init();
            let buf = std::slice::from_raw_parts_mut(&mut mem as *mut S as *mut u8, size);
            match self.read_exact(buf) {
                Ok(()) => Ok(mem),
                Err(e) => {
                    std::mem::forget(mem);
                    Err(e)
                }
            }
        }
    }

    fn assert_end(&mut self) -> crate::Result<()> {
        let mut buf = [0; 1];
        match self.read(&mut buf)? {
            0 => Ok(()),
            _ => Err(AssertionError("Expected all data to be read".to_owned()))?,
        }
    }

    fn read_string<T>(&mut self, offset: &mut T) -> crate::Result<String>
    where
        T: std::ops::AddAssign + FromUsize + std::fmt::Display + Copy,
    {
        let count = self.read_u32()? as usize;
        *offset += T::from_usize(4usize);
        let mut buf = vec![0u8; count];
        self.read_exact(&mut buf)?;
        let value = assert_utf8("value", *offset, || str_from_c_sized(&buf))?;
        *offset += T::from_usize(count);
        Ok(value.to_owned())
    }
}

pub trait WriteHelper {
    fn write_u32(&mut self, value: u32) -> Result<()>;
    fn write_i32(&mut self, value: i32) -> Result<()>;
    fn write_f32(&mut self, value: f32) -> Result<()>;
    fn write_u16(&mut self, value: u16) -> Result<()>;
    fn write_i16(&mut self, value: i16) -> Result<()>;
    fn write_struct<S>(&mut self, value: &S) -> Result<()>;
    fn write_string(&mut self, value: &str) -> crate::Result<()>;
}

impl<W> WriteHelper for W
where
    W: Write,
{
    fn write_u32(&mut self, value: u32) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_i32(&mut self, value: i32) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_f32(&mut self, value: f32) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_u16(&mut self, value: u16) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_i16(&mut self, value: i16) -> Result<()> {
        let buf = value.to_le_bytes();
        self.write_all(&buf)
    }

    fn write_struct<S>(&mut self, value: &S) -> Result<()> {
        let size = std::mem::size_of::<S>();
        let buf = unsafe { std::slice::from_raw_parts(value as *const S as *const u8, size) };
        self.write_all(buf)
    }

    fn write_string(&mut self, value: &str) -> crate::Result<()> {
        let buf = value.as_bytes();
        let count = buf.len() as u32;
        self.write_u32(count)?;
        self.write_all(&buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::str_to_c_padded;
    use std::io::Cursor;

    trait ReadAll {
        fn read_all(&mut self) -> Vec<u8>;
    }

    impl ReadAll for Cursor<Vec<u8>> {
        fn read_all(&mut self) -> Vec<u8> {
            let mut buf = Vec::new();
            self.set_position(0);
            self.read_to_end(&mut buf).unwrap();
            self.set_position(0);
            buf
        }
    }

    #[test]
    fn u32_roundtrip() {
        let expected = vec![0xEF, 0xBE, 0xAD, 0xDE];
        let mut input = Cursor::new(expected.clone());
        assert_eq!(3735928559, input.read_u32().unwrap());

        let mut output = Cursor::new(vec![]);
        output.write_u32(3735928559).unwrap();
        assert_eq!(expected, output.read_all());
    }

    #[test]
    fn i32_roundtrip() {
        let expected = vec![0xEF, 0xBE, 0xAD, 0xDE];
        let mut input = Cursor::new(expected.clone());
        assert_eq!(-559038737, input.read_i32().unwrap());

        let mut output = Cursor::new(vec![]);
        output.write_i32(-559038737).unwrap();
        assert_eq!(expected, output.read_all());
    }

    #[test]
    fn f32_roundtrip() {
        let expected = -1.0f32;
        let mut cursor = Cursor::new(vec![]);
        cursor.write_f32(expected).unwrap();
        cursor.set_position(0);
        let actual = cursor.read_f32().unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn u16_roundtrip() {
        let expected = vec![0xEF, 0xBE];
        let mut input = Cursor::new(expected.clone());
        assert_eq!(48879, input.read_u16().unwrap());

        let mut output = Cursor::new(vec![]);
        output.write_u16(48879).unwrap();
        assert_eq!(expected, output.read_all());
    }

    #[test]
    fn i16_roundtrip() {
        let expected = vec![0xEF, 0xBE];
        let mut input = Cursor::new(expected.clone());
        assert_eq!(-16657, input.read_i16().unwrap());

        let mut output = Cursor::new(vec![]);
        output.write_i16(-16657).unwrap();
        assert_eq!(expected, output.read_all());
    }

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    struct TestStruct {
        name: [u8; 32],
        int: u32,
    }

    #[test]
    fn struct_roundtrip() {
        let mut name = [0u8; 32];
        str_to_c_padded("Hello World", &mut name);
        let expected = TestStruct {
            name,
            int: 3735928559,
        };

        let mut cursor = Cursor::new(vec![]);
        cursor.write_struct(&expected).unwrap();
        assert_eq!(std::mem::size_of::<TestStruct>() as u64, cursor.position());

        cursor.set_position(0);
        let actual: TestStruct = cursor.read_struct().unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn string_roundtrip() {
        let expected = "Hello World".to_owned();
        let mut cursor = Cursor::new(vec![]);
        cursor.write_string(&expected).unwrap();
        cursor.set_position(0);
        let mut offset = 0;
        let actual = cursor.read_string(&mut offset).unwrap();
        assert_eq!(expected, actual);
        assert_eq!(offset, expected.len() + 4);
    }
}
