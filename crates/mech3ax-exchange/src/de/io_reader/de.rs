use super::private::EnumType;
use super::IoReader;
use crate::error::{Error, ErrorCode, Result};
use serde::de;
use std::io::Read;

macro_rules! err_unsupported {
    () => {
        Err(Error::new(ErrorCode::UnsupportedType))
    };
}

macro_rules! serde_unsupported {
    ($method:ident) => {
        fn $method<V>(self, _visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            err_unsupported!()
        }
    };
}

impl<'a, 'de: 'a, R: Read> de::Deserializer<'de> for &'a mut IoReader<R> {
    type Error = Error;

    fn is_human_readable(&self) -> bool {
        false
    }

    serde_unsupported!(deserialize_i64);
    serde_unsupported!(deserialize_u64);
    serde_unsupported!(deserialize_f64);
    serde_unsupported!(deserialize_char);
    serde_unsupported!(deserialize_any);

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i8(self.read_i8()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(self.read_i16()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(self.read_i32()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(self.read_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(self.read_u16()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.read_u32()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.read_f32()?)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.read_bool()?)
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.read_str()?)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_byte_buf(self.read_bytes()?)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.read_option()? {
            true => visitor.visit_some(self),
            false => visitor.visit_none(),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len = self.read_seq_sized()?;
        visitor.visit_seq(SizedSeqAccess {
            deserializer: self,
            len,
        })
    }

    fn deserialize_tuple<V>(self, _tuple_len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // As is done here, serializers are encouraged to treat newtype structs
        // as insignificant wrappers around the data they contain.
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len = self.read_struct(name)?;
        visitor.visit_map(SizedMapAccess {
            deserializer: self,
            len,
        })
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let (enum_type, variant_index) = self.read_enum(name)?;
        let variant = variants
            .get(variant_index as usize)
            .ok_or_else(|| Error::new(ErrorCode::InvalidVariant))?;
        match enum_type {
            EnumType::Unit => visitor.visit_enum(EnumUnit { variant }),
            EnumType::NewType => visitor.visit_enum(EnumNewType {
                deserializer: self,
                variant,
            }),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }
}

struct SizedSeqAccess<'a, R> {
    deserializer: &'a mut IoReader<R>,
    len: usize,
}

impl<'a, 'de: 'a, R: Read> de::SeqAccess<'de> for SizedSeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.len > 0 {
            self.len -= 1;
            seed.deserialize(&mut *self.deserializer).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct SizedMapAccess<'a, R> {
    deserializer: &'a mut IoReader<R>,
    len: usize,
}

impl<'a, 'de: 'a, R: Read> de::MapAccess<'de> for SizedMapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.len > 0 {
            self.len -= 1;
            seed.deserialize(&mut *self.deserializer).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.deserializer)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct EnumUnit {
    variant: &'static str,
}

impl<'de> de::EnumAccess<'de> for EnumUnit {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let deserializer = de::value::StrDeserializer::new(self.variant);
        let val = seed.deserialize(deserializer)?;
        Ok((val, self))
    }
}

impl<'de> de::VariantAccess<'de> for EnumUnit {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(Error::new(ErrorCode::InvalidVariant))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }
}

struct EnumNewType<'a, R> {
    deserializer: &'a mut IoReader<R>,
    variant: &'static str,
}

impl<'a, 'de: 'a, R: Read> de::EnumAccess<'de> for EnumNewType<'a, R> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let deserializer = de::value::StrDeserializer::new(self.variant);
        let val = seed.deserialize(deserializer)?;
        Ok((val, self))
    }
}

impl<'a, 'de: 'a, R: Read> de::VariantAccess<'de> for EnumNewType<'a, R> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::new(ErrorCode::InvalidVariant))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.deserializer)
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        err_unsupported!()
    }
}
