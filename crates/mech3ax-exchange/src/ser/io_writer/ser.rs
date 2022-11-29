use super::IoWriter;
use crate::error::{Error, ErrorCode, Result};
use serde::{ser, Serialize};
use std::io::Write;

macro_rules! err_unsupported {
    () => {
        Err(Error::new(ErrorCode::UnsupportedType))
    };
}

macro_rules! serde_unsupported {
    ($method:ident, $type:ty) => {
        fn $method(self, _value: $type) -> Result<()> {
            err_unsupported!()
        }
    };
}

impl<'a, W: Write> ser::Serializer for &'a mut IoWriter<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }

    serde_unsupported!(serialize_i64, i64);
    serde_unsupported!(serialize_u64, u64);
    serde_unsupported!(serialize_f64, f64);
    serde_unsupported!(serialize_char, char);

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.write_i8(v)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.write_i16(v)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.write_i32(v)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.write_u8(v)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.write_u16(v)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.write_u32(v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.write_f32(v)
    }

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.write_bool(v)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_str(v)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.write_bytes(v)
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.write_none()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_some()?;
        value.serialize(&mut *self)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.write_seq_unsized(len)?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        err_unsupported!()
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        err_unsupported!()
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        err_unsupported!()
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        err_unsupported!()
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        // As is done here, serializers are encouraged to treat newtype structs as
        // insignificant wrappers around the data they contain.
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        err_unsupported!()
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.write_struct(len)?;
        Ok(self)
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.write_enum_unit(variant_index)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_enum_newtype(variant_index)?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        err_unsupported!()
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        err_unsupported!()
    }
}

impl<'a, W: Write> ser::SerializeSeq for &'a mut IoWriter<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeStruct for &'a mut IoWriter<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_str(key)?;
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTuple for &'a mut IoWriter<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        err_unsupported!()
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTupleStruct for &'a mut IoWriter<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        err_unsupported!()
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeMap for &'a mut IoWriter<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        err_unsupported!()
    }

    #[inline]
    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        err_unsupported!()
    }

    #[inline]
    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, _key: &K, _value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        err_unsupported!()
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeTupleVariant for &'a mut IoWriter<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        err_unsupported!()
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> ser::SerializeStructVariant for &'a mut IoWriter<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        err_unsupported!()
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}
