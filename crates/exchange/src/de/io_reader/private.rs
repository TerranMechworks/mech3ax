use crate::constants::TypeMap;
use crate::error::{Error, ErrorCode, Result};
use std::io::Read;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumType {
    Unit,
    NewType,
}

#[derive(Debug, Clone)]
pub struct IoReader<R> {
    inner: R,
}

impl<R> IoReader<R> {
    #[inline]
    pub const fn new(reader: R) -> Self {
        Self { inner: reader }
    }
}

macro_rules! read_basic_type {
    ($name:ident, $ty:ty, $type_map:expr) => {
        #[inline]
        pub fn $name(&mut self) -> Result<$ty> {
            self.expect_type($type_map)?;
            let mut buf = [0; std::mem::size_of::<$ty>()];
            self.read_all(&mut buf)?;
            Ok(<$ty>::from_le_bytes(buf))
        }
    };
}

impl<R: Read> IoReader<R> {
    #[inline]
    fn read_all(&mut self, buf: &mut [u8]) -> Result<()> {
        self.inner.read_exact(buf).map_err(Error::io)
    }

    #[inline]
    fn read_type(&mut self) -> Result<TypeMap> {
        let mut buf = [0; 4];
        self.read_all(&mut buf)?;
        TypeMap::from_u32(u32::from_le_bytes(buf)).ok_or_else(|| Error::new(ErrorCode::InvalidType))
    }

    #[inline]
    fn read_usize(&mut self) -> Result<usize> {
        let mut buf = [0; 8];
        self.read_all(&mut buf)?;
        Ok(usize::from_le_bytes(buf))
    }

    #[inline]
    fn read_string_raw(&mut self) -> Result<String> {
        let len = self.read_usize()?;
        let mut buf = vec![0u8; len];
        self.read_all(&mut buf)?;
        String::from_utf8(buf).map_err(|e| Error::new(ErrorCode::InvalidUtf8(e)))
    }

    #[inline]
    fn expect_type(&mut self, expected: TypeMap) -> Result<()> {
        let actual = self.read_type()?;
        if actual == expected {
            Ok(())
        } else {
            Err(Error::unexpected_type(expected, actual))
        }
    }

    read_basic_type!(read_i8, i8, TypeMap::I8);
    read_basic_type!(read_i16, i16, TypeMap::I16);
    read_basic_type!(read_i32, i32, TypeMap::I32);
    read_basic_type!(read_u8, u8, TypeMap::U8);
    read_basic_type!(read_u16, u16, TypeMap::U16);
    read_basic_type!(read_u32, u32, TypeMap::U32);
    read_basic_type!(read_f32, f32, TypeMap::F32);

    #[inline]
    pub fn read_bool(&mut self) -> Result<bool> {
        let actual = self.read_type()?;
        match actual {
            TypeMap::BoolTrue => Ok(true),
            TypeMap::BoolFalse => Ok(false),
            actual => Err(Error::unexpected_type("boolean", actual)),
        }
    }

    #[inline]
    pub fn read_str(&mut self) -> Result<String> {
        self.expect_type(TypeMap::Str)?;
        self.read_string_raw()
    }

    #[inline]
    pub fn read_bytes(&mut self) -> Result<Vec<u8>> {
        self.expect_type(TypeMap::Bytes)?;
        let len = self.read_usize()?;
        let mut buf = vec![0u8; len];
        self.read_all(&mut buf)?;
        Ok(buf)
    }

    #[inline]
    pub fn read_option(&mut self) -> Result<bool> {
        let actual = self.read_type()?;
        match actual {
            TypeMap::Some => Ok(true),
            TypeMap::None => Ok(false),
            actual => Err(Error::unexpected_type("option", actual)),
        }
    }

    #[inline]
    pub fn read_seq_sized(&mut self) -> Result<usize> {
        self.expect_type(TypeMap::Seq)?;
        self.read_usize()
    }

    #[inline]
    pub fn read_struct(&mut self) -> Result<usize> {
        self.expect_type(TypeMap::Struct)?;
        self.read_usize()
    }

    #[inline]
    fn read_enum_variant(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_all(&mut buf)?;
        let variant_index = u32::from_le_bytes(buf);
        Ok(variant_index)
    }

    #[inline]
    pub fn read_enum(&mut self) -> Result<(EnumType, u32)> {
        let actual = self.read_type()?;
        let enum_type = match actual {
            TypeMap::EnumUnit => Ok(EnumType::Unit),
            TypeMap::EnumNewType => Ok(EnumType::NewType),
            actual => Err(Error::unexpected_type("enum", actual)),
        }?;
        let variant_index = self.read_enum_variant()?;
        Ok((enum_type, variant_index))
    }
}
