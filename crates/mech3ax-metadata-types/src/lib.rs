//! Types for API type metadata.
//!
//! All public API types must have metadata, so that type definitions and JSON
//! serialization/deserialization code can be automatically generated. For now,
//! this is specifically C#.
use time::OffsetDateTime;

/// Base types that can always be used, without special declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeInfoBase {
    /// Rust: bool, C#: bool
    Bool,
    /// Rust: u8, C#: byte
    U8,
    /// Rust: u16, C#: ushort
    U16,
    /// Rust: u32, C#: uint
    U32,
    /// Rust: i8, C#: sbyte
    I8,
    /// Rust: i16, C#: short
    I16,
    /// Rust: i32, C#: int
    I32,
    /// Rust: f32, C#: float
    F32,
    /// Rust: String, C#: string
    String,
    /// Rust: OffsetDateTime, C#: DateTime
    DateTime,
}

/// A `Vec<T>`/`List<T>` type.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfoVec {
    pub inner: &'static TypeInfo,
}

/// An `Option<T>`/`Nullable<T>` type.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfoOption {
    pub inner: &'static TypeInfo,
}

/// An enum type, with string variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfoEnum {
    pub name: &'static str,
    pub variants: &'static [&'static str],
    pub module_path: &'static str,
}

/// A discriminant union type.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfoUnion {
    pub name: &'static str,
    pub variants: &'static [(&'static str, Option<&'static TypeInfo>)],
    pub module_path: &'static str,
}

/// A janky way of specifying whether the struct should be a reference type
/// (C# `class`) or a value type (C# `struct`).
///
/// In other words, a leaky abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeSemantic {
    Val,
    Ref,
}

/// A limit set of known default value handling behaviours.
///
/// * `Normal` indicates values must be present.
/// * `OptionIsNone` indicates `None` values can be omitted from serialization,
///   and implied during deserialization.
/// * `BoolFalse` indicates `false` values can be omitted from serialization,
///   and implied during deserialization.
/// * `BoolTrue` indicates `true` values can be omitted from serialization,
///   and implied during deserialization.
/// * `PointerZero` indicates `0` values can be omitted from serialization,
///   and implied during deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultHandling {
    Normal,
    OptionIsNone,
    BoolFalse,
    BoolTrue,
    PointerZero,
}

/// Information for a field on a struct.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfoStructField {
    pub name: &'static str,
    pub type_info: &'static TypeInfo,
    pub default: DefaultHandling,
}

/// A (Rust) struct type.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfoStruct {
    pub name: &'static str,
    pub semantic: TypeSemantic,
    pub generics: Option<&'static [(&'static TypeInfo, &'static str)]>,
    pub fields: &'static [TypeInfoStructField],
    pub module_path: &'static str,
    pub partial: bool,
}

/// A type.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    // leaf type
    Base(TypeInfoBase),
    // leaf type
    Enum(TypeInfoEnum),
    Vec(TypeInfoVec),
    Option(TypeInfoOption),
    Struct(TypeInfoStruct),
    Union(TypeInfoUnion),
}

/// A trait that all types must implement to be eligible for use in the public
/// API, metadata generation, and C# code auto-generation.
pub trait DerivedMetadata {
    const TYPE_INFO: &'static TypeInfo;
}

macro_rules! base_type {
    ($ty:ty, $base:expr) => {
        impl DerivedMetadata for $ty {
            const TYPE_INFO: &'static TypeInfo = &TypeInfo::Base($base);
        }
    };
}

base_type!(bool, TypeInfoBase::Bool);
base_type!(u8, TypeInfoBase::U8);
base_type!(u16, TypeInfoBase::U16);
base_type!(u32, TypeInfoBase::U32);
base_type!(i8, TypeInfoBase::I8);
base_type!(i16, TypeInfoBase::I16);
base_type!(i32, TypeInfoBase::I32);
base_type!(f32, TypeInfoBase::F32);
base_type!(String, TypeInfoBase::String);
base_type!(OffsetDateTime, TypeInfoBase::DateTime);

impl<Inner: DerivedMetadata> DerivedMetadata for Vec<Inner> {
    const TYPE_INFO: &'static TypeInfo = &TypeInfo::Vec(TypeInfoVec {
        inner: Inner::TYPE_INFO,
    });
}

impl<Inner: DerivedMetadata> DerivedMetadata for Option<Inner> {
    const TYPE_INFO: &'static TypeInfo = &TypeInfo::Option(TypeInfoOption {
        inner: Inner::TYPE_INFO,
    });
}
