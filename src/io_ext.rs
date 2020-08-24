use std::io::{Read, Result, Write};
use std::mem::MaybeUninit;

pub trait ReadHelper {
    fn read_u32(&mut self) -> Result<u32>;
    fn read_i32(&mut self) -> Result<i32>;
    fn read_struct<S>(&mut self) -> Result<S>;
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
}

pub trait WriteHelper {
    fn write_u32(&mut self, value: u32) -> Result<()>;
    fn write_i32(&mut self, value: i32) -> Result<()>;
    fn write_struct<S>(&mut self, value: &S) -> Result<()>;
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

    fn write_struct<S>(&mut self, value: &S) -> Result<()> {
        let size = std::mem::size_of::<S>();
        let buf = unsafe { std::slice::from_raw_parts(value as *const S as *const u8, size) };
        self.write_all(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::str_to_c;
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

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    struct TestStruct {
        name: [u8; 32],
        int: u32,
    }

    #[test]
    fn struct_roundtrip() {
        let mut name = [0u8; 32];
        str_to_c("Hello World", &mut name);
        let expected = TestStruct {
            name,
            int: 3735928559,
        };

        let mut cursor = Cursor::new(vec![]);
        cursor.write_struct::<TestStruct>(&expected).unwrap();
        assert_eq!(std::mem::size_of::<TestStruct>() as u64, cursor.position());

        cursor.set_position(0);
        let actual = cursor.read_struct::<TestStruct>().unwrap();
        assert_eq!(expected, actual);
    }
}
