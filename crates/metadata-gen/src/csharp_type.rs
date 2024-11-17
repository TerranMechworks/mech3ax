use mech3ax_metadata_types::TypeInfoBase;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Write as _;

/// The C# type kind that is also null-aware.
///
/// Value and reference are supported, while output doesn't make sense.
///
/// # See also
///
/// https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    /// Value type, non-null
    Val,
    /// Value type, nullable
    ValNull,
    /// Reference type, non-null
    Ref,
    /// Reference type, nullable
    RefNull,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerializeType {
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
    ValOption(Box<SerializeType>),
    RefOption(Box<SerializeType>),
    String,
    Bytes,
    Vec(Box<SerializeType>),
    Class(String),
    Struct(String),
    Enum(String),
    Union(String),
    Generic(&'static str),
}

impl SerializeType {
    fn make_ser(&self, s: &mut String) {
        use SerializeType::*;
        match self {
            U8 => s.push_str("((Action<byte>)s.SerializeU8)"),
            U16 => s.push_str("((Action<ushort>)s.SerializeU16)"),
            U32 => s.push_str("((Action<uint>)s.SerializeU32)"),
            U64 => s.push_str("((Action<uint>)s.SerializeU64)"),
            I8 => s.push_str("((Action<sbyte>)s.SerializeI8)"),
            I16 => s.push_str("((Action<short>)s.SerializeI16)"),
            I32 => s.push_str("((Action<int>)s.SerializeI32)"),
            F32 => s.push_str("((Action<float>)s.SerializeF32)"),
            Bool => s.push_str("((Action<bool>)s.SerializeBool)"),
            DateTime => s.push_str("((Action<DateTime>)s.SerializeDateTime)"),
            String => s.push_str("((Action<string>)s.SerializeString)"),
            Bytes => s.push_str("((Action<byte[]>)s.SerializeBytes)"),
            ValOption(inner) => {
                s.push_str("s.SerializeValOption(");
                inner.make_ser(s);
                s.push(')');
            }
            RefOption(inner) => {
                s.push_str("s.SerializeRefOption(");
                inner.make_ser(s);
                s.push(')');
            }
            Vec(inner) => {
                s.push_str("s.SerializeVec(");
                inner.make_ser(s);
                s.push(')');
            }
            Class(full_type) | Union(full_type) => {
                write!(s, "s.Serialize({}.Converter)", full_type).unwrap()
            }
            Struct(full_type) | Enum(full_type) => {
                write!(s, "s.Serialize({}Converter.Converter)", full_type).unwrap()
            }
            Generic(full_type) => write!(s, "s.SerializeGeneric<{}>()", full_type).unwrap(),
        }
    }

    pub fn make_serialize(&self) -> String {
        let mut s = String::new();
        self.make_ser(&mut s);
        s
    }

    fn make_de(&self, s: &mut String) {
        use SerializeType::*;
        match self {
            U8 => s.push_str("d.DeserializeU8"),
            U16 => s.push_str("d.DeserializeU16"),
            U32 => s.push_str("d.DeserializeU32"),
            U64 => s.push_str("d.DeserializeU64"),
            I8 => s.push_str("d.DeserializeI8"),
            I16 => s.push_str("d.DeserializeI16"),
            I32 => s.push_str("d.DeserializeI32"),
            F32 => s.push_str("d.DeserializeF32"),
            Bool => s.push_str("d.DeserializeBool"),
            DateTime => s.push_str("d.DeserializeDateTime"),
            String => s.push_str("d.DeserializeString"),
            Bytes => s.push_str("d.DeserializeBytes"),
            ValOption(inner) => {
                s.push_str("d.DeserializeValOption(");
                inner.make_de(s);
                s.push(')');
            }
            RefOption(inner) => {
                s.push_str("d.DeserializeRefOption(");
                inner.make_de(s);
                s.push(')');
            }
            Vec(inner) => {
                s.push_str("d.DeserializeVec(");
                inner.make_de(s);
                s.push(')');
            }
            Class(full_type) | Union(full_type) => {
                write!(s, "d.Deserialize({}.Converter)", full_type).unwrap()
            }
            Struct(full_type) | Enum(full_type) => {
                write!(s, "d.Deserialize({}Converter.Converter)", full_type).unwrap()
            }
            Generic(full_type) => write!(s, "d.DeserializeGeneric<{}>()", full_type).unwrap(),
        }
    }

    pub fn make_deserialize(&self) -> String {
        let mut s = String::new();
        self.make_de(&mut s);
        s
    }
}

/// A C# type consisting of a name and kind.
///
/// This is used for type resolution, and should be cheap to construct.
///
/// For generic types, the generic parameters must be part of the name, for
/// example "Foo<T>".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpType {
    /// The fully-qualified C# type name.
    pub name: Cow<'static, str>,
    pub kind: TypeKind,
    pub generics: Option<HashSet<&'static str>>,
    pub serde: SerializeType,
}

impl From<&TypeInfoBase> for CSharpType {
    fn from(base: &TypeInfoBase) -> Self {
        use TypeInfoBase::*;
        let (name, kind, serde) = match base {
            Bool => ("bool", TypeKind::Val, SerializeType::Bool),
            U8 => ("byte", TypeKind::Val, SerializeType::U8),
            U16 => ("ushort", TypeKind::Val, SerializeType::U16),
            U32 => ("uint", TypeKind::Val, SerializeType::U32),
            U64 => ("ulong", TypeKind::Val, SerializeType::U64),
            I8 => ("sbyte", TypeKind::Val, SerializeType::I8),
            I16 => ("short", TypeKind::Val, SerializeType::I16),
            I32 => ("int", TypeKind::Val, SerializeType::I32),
            F32 => ("float", TypeKind::Val, SerializeType::F32),
            DateTime => ("System.DateTime", TypeKind::Val, SerializeType::DateTime),
            String => ("string", TypeKind::Ref, SerializeType::String),
        };
        Self {
            name: Cow::Borrowed(name),
            kind,
            generics: None,
            serde,
        }
    }
}

impl CSharpType {
    pub fn is_byte(&self) -> bool {
        self.name == "byte"
            && self.kind == TypeKind::Val
            && self.generics.is_none()
            && self.serde == SerializeType::U8
    }

    pub fn byte_vec() -> Self {
        Self {
            name: Cow::Borrowed("byte[]"),
            kind: TypeKind::Val,
            generics: None,
            serde: SerializeType::Bytes,
        }
    }

    /// Convert a type into an option/nullable type.
    pub fn option(mut inner: Self) -> Self {
        match inner.kind {
            TypeKind::Val => {
                inner.name = Cow::Owned(format!("{}?", inner.name));
                inner.kind = TypeKind::ValNull;
                inner.serde = SerializeType::ValOption(Box::new(inner.serde));
            }
            TypeKind::Ref => {
                inner.name = Cow::Owned(format!("{}?", inner.name));
                inner.kind = TypeKind::RefNull;
                inner.serde = SerializeType::RefOption(Box::new(inner.serde));
            }
            TypeKind::ValNull => {
                eprintln!("WARNING: doubly-nullable value type `{}`", inner.name);
            }
            TypeKind::RefNull => {
                eprintln!("WARNING: doubly-nullable ref type `{}`", inner.name);
            }
        };
        inner
    }

    /// Convert a type into a vec/list type.
    pub fn vec(mut inner: Self) -> Self {
        inner.name = Cow::Owned(format!("System.Collections.Generic.List<{}>", inner.name));
        // a list is always a reference type, and has it's own nullability,
        // independent of the inner type.
        inner.kind = TypeKind::Ref;
        inner.serde = SerializeType::Vec(Box::new(inner.serde));
        inner
    }
}
