use crate::constants::TypeMap;
use crate::error::{Error, ErrorCode, Result};
use std::io::Write;

#[derive(Debug, Clone)]
pub struct IoWriter<W> {
    inner: W,
}

impl<W> IoWriter<W> {
    #[inline]
    pub const fn new(writer: W) -> Self {
        Self { inner: writer }
    }
}

macro_rules! write_basic_type {
    ($name:ident, $ty:ty, $type_map:expr) => {
        #[inline]
        pub fn $name(&mut self, v: $ty) -> Result<()> {
            self.write_type($type_map)?;
            self.write_all(&v.to_le_bytes())
        }
    };
}

impl<W: Write> IoWriter<W> {
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.inner.write_all(buf).map_err(Error::io)
    }

    #[inline]
    fn write_type(&mut self, ty: TypeMap) -> Result<()> {
        let buf = ty.to_u32().to_le_bytes();
        self.write_all(&buf)
    }

    #[inline]
    fn write_usize(&mut self, v: usize) -> Result<()> {
        let buf: [u8; 8] = v.to_le_bytes();
        self.write_all(&buf)
    }

    #[inline]
    fn write_string_raw(&mut self, s: &str) -> Result<()> {
        let v = s.as_bytes();
        let len = v.len();
        self.write_usize(len)?;
        self.write_all(v)
    }

    write_basic_type!(write_i8, i8, TypeMap::I8);
    write_basic_type!(write_i16, i16, TypeMap::I16);
    write_basic_type!(write_i32, i32, TypeMap::I32);
    write_basic_type!(write_u8, u8, TypeMap::U8);
    write_basic_type!(write_u16, u16, TypeMap::U16);
    write_basic_type!(write_u32, u32, TypeMap::U32);
    write_basic_type!(write_f32, f32, TypeMap::F32);

    #[inline]
    pub fn write_bool(&mut self, v: bool) -> Result<()> {
        self.write_type(if v {
            TypeMap::BoolTrue
        } else {
            TypeMap::BoolFalse
        })
    }

    #[inline]
    pub fn write_str(&mut self, v: &str) -> Result<()> {
        self.write_type(TypeMap::Str)?;
        self.write_string_raw(v)
    }

    #[inline]
    pub fn write_bytes(&mut self, v: &[u8]) -> Result<()> {
        let len = v.len();
        self.write_type(TypeMap::Bytes)?;
        self.write_usize(len)?;
        self.write_all(v)
    }

    #[inline]
    pub fn write_none(&mut self) -> Result<()> {
        self.write_type(TypeMap::None)
    }

    #[inline]
    pub fn write_some(&mut self) -> Result<()> {
        self.write_type(TypeMap::Some)
    }

    #[inline]
    pub fn write_seq_unsized(&mut self, len: Option<usize>) -> Result<()> {
        let len = len.ok_or_else(|| Error::new(ErrorCode::SequenceMustHaveLength))?;
        self.write_seq_sized(len)
    }

    #[inline]
    pub fn write_seq_sized(&mut self, len: usize) -> Result<()> {
        self.write_type(TypeMap::Seq)?;
        self.write_usize(len)
    }

    #[inline]
    pub fn write_struct(&mut self, name: &'static str, len: usize) -> Result<()> {
        self.write_type(TypeMap::Struct)?;
        self.write_string_raw(name)?;
        self.write_usize(len)
    }

    #[inline]
    fn write_enum_variant(&mut self, variant_index: u32) -> Result<()> {
        self.write_all(&variant_index.to_le_bytes())
    }

    #[inline]
    pub fn write_enum_unit(&mut self, name: &'static str, variant_index: u32) -> Result<()> {
        self.write_type(TypeMap::EnumUnit)?;
        self.write_string_raw(name)?;
        self.write_enum_variant(variant_index)
    }

    #[inline]
    pub fn write_enum_newtype(&mut self, name: &'static str, variant_index: u32) -> Result<()> {
        self.write_type(TypeMap::EnumNewType)?;
        self.write_string_raw(name)?;
        self.write_enum_variant(variant_index)
    }

    pub fn finish(mut self) -> Result<W> {
        self.inner.flush().map_err(Error::io)?;
        Ok(self.inner)
    }
}
