use mech3ax_metadata_types::TypeInfoBase;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SerializeType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    F32,
    Bool,
    DateTime,
    String,
    Bytes,
    Option(Box<SerializeType>),
    Vec(Box<SerializeType>),
    Class(&'static str),
    Enum(&'static str),
    Flags(&'static str),
    Union(&'static str),
}

impl SerializeType {
    fn make_ser(&self, s: &mut String) {
        use SerializeType::*;
        match self {
            U8 => s.push_str("s.serialize_u8"),
            U16 => s.push_str("s.serialize_u16"),
            U32 => s.push_str("s.serialize_u32"),
            U64 => s.push_str("s.serialize_u64"),
            I8 => s.push_str("s.serialize_i8"),
            I16 => s.push_str("s.serialize_i16"),
            I32 => s.push_str("s.serialize_i32"),
            F32 => s.push_str("s.serialize_f32"),
            Bool => s.push_str("s.serialize_bool"),
            DateTime => s.push_str("s.serialize_datetime"),
            String => s.push_str("s.serialize_str"),
            Bytes => s.push_str("s.serialize_bytes"),
            Option(inner) => {
                s.push_str("s.serialize_option(");
                inner.make_ser(s);
                s.push(')');
            }
            Vec(inner) => {
                s.push_str("s.serialize_vec(");
                inner.make_ser(s);
                s.push(')');
            }
            Class(type_name) | Enum(type_name) | Flags(type_name) | Union(type_name) => {
                s.push_str("s.serialize(");
                s.push_str(type_name);
                s.push(')');
            }
        }
    }

    pub(crate) fn make_serialize(&self) -> String {
        let mut s = String::new();
        self.make_ser(&mut s);
        s
    }

    fn make_de(&self, s: &mut String) {
        use SerializeType::*;
        match self {
            U8 => s.push_str("d.deserialize_u8"),
            U16 => s.push_str("d.deserialize_u16"),
            U32 => s.push_str("d.deserialize_u32"),
            U64 => s.push_str("d.deserialize_u64"),
            I8 => s.push_str("d.deserialize_i8"),
            I16 => s.push_str("d.deserialize_i16"),
            I32 => s.push_str("d.deserialize_i32"),
            F32 => s.push_str("d.deserialize_f32"),
            Bool => s.push_str("d.deserialize_bool"),
            DateTime => s.push_str("d.deserialize_datetime"),
            String => s.push_str("d.deserialize_str"),
            Bytes => s.push_str("d.deserialize_bytes"),
            Option(inner) => {
                s.push_str("d.deserialize_option(");
                inner.make_de(s);
                s.push(')');
            }
            Vec(inner) => {
                s.push_str("d.deserialize_vec(");
                inner.make_de(s);
                s.push(')');
            }
            Class(type_name) | Enum(type_name) | Flags(type_name) | Union(type_name) => {
                s.push_str("d.deserialize(");
                s.push_str(type_name);
                s.push(')');
            }
        }
    }

    pub(crate) fn make_deserialize(&self) -> String {
        let mut s = String::new();
        self.make_de(&mut s);
        s
    }
}

/// A Python type.
///
/// This is used for type resolution, and should be cheap to construct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PythonType {
    pub(crate) name: Cow<'static, str>,
    pub(crate) import: Option<String>,
    pub(crate) nullable: bool,
    pub(crate) serde: SerializeType,
}

impl From<&TypeInfoBase> for PythonType {
    fn from(base: &TypeInfoBase) -> Self {
        use TypeInfoBase::*;
        let (name, namespace, serde) = match base {
            Bool => ("bool", None, SerializeType::Bool),
            U8 => ("int", None, SerializeType::U8),
            U16 => ("int", None, SerializeType::U16),
            U32 => ("int", None, SerializeType::U32),
            U64 => ("int", None, SerializeType::U64),
            I8 => ("int", None, SerializeType::I8),
            I16 => ("int", None, SerializeType::I16),
            I32 => ("int", None, SerializeType::I32),
            F32 => ("float", None, SerializeType::F32),
            String => ("str", None, SerializeType::String),
            DateTime => (
                "datetime",
                Some("from datetime import datetime".to_string()),
                SerializeType::DateTime,
            ),
        };
        Self {
            name: Cow::Borrowed(name),
            import: namespace,
            nullable: false,
            serde,
        }
    }
}

impl PythonType {
    pub(crate) fn is_byte(&self) -> bool {
        self.serde == SerializeType::U8
            && self.import.is_none()
            && !self.nullable
            && self.name == "int"
    }

    pub(crate) fn byte_vec() -> Self {
        Self {
            name: Cow::Borrowed("bytes"),
            import: None,
            nullable: false,
            serde: SerializeType::Bytes,
        }
    }

    /// Convert a type into an option/nullable type.
    pub(crate) fn option(mut inner: Self) -> Self {
        match inner.nullable {
            false => {
                inner.name = Cow::Owned(format!("{} | None", inner.name));
                inner.nullable = true;
                inner.serde = SerializeType::Option(Box::new(inner.serde));
            }
            true => {
                eprintln!("WARNING: doubly-nullable type `{}`", inner.name);
            }
        };
        inner
    }

    /// Convert a type into a vec/list type.
    pub(crate) fn vec(mut inner: Self) -> Self {
        inner.name = Cow::Owned(format!("list[{}]", inner.name));
        // a list has it's own nullability, independent of the inner type
        inner.nullable = false;
        inner.serde = SerializeType::Vec(Box::new(inner.serde));
        inner
    }
}
