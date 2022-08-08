#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimpleType {
    /// Rust: bool, C#: bool
    Bool,
    /// Rust: u8, C#: byte
    U8,
    /// Rust: u16, C#: ushort
    U16,
    /// Rust: u32, C#: uint
    U32,
    // /// Rust: u64, C#: ulong
    // U64,
    /// Rust: i8, C#: sbyte
    I8,
    /// Rust: i16, C#: short
    I16,
    /// Rust: i32, C#: int
    I32,
    // /// Rust: i64, C#: long
    // I64,
    /// Rust: f32, C#: float
    F32,
    // /// Rust: f64, C#: double
    // F64,
    /// Rust: String, C#: string
    String,
    /// Rust: Vec<u8>, C#: byte[]
    Bytes,
    /// Rust: OffsetDateTime, C#: DateTime
    DateTime,
}

#[derive(Debug)]
pub enum ComplexTypeOwned {
    Simple(SimpleType),
    /// Assumed to be another structure that is defined
    Struct(String),
    /// Rust: Option<T>, C#: Nullable<T>
    Option(Box<ComplexTypeOwned>),
    /// Rust: Vec<T>, C#: List<T>
    Vec(Box<ComplexTypeOwned>),
}

#[derive(Debug)]
pub enum ComplexType<'a> {
    Simple(SimpleType),
    /// Assumed to be another structure that is defined
    Struct(&'a str),
    /// Rust: Option<T>, C#: Nullable<T>
    Option(&'a ComplexType<'a>),
    /// Rust: Vec<T>, C#: List<T>
    Vec(&'a ComplexType<'a>),
}

#[derive(Debug)]
pub struct TypeInfoOwned {
    pub name: String,
    pub ty: ComplexTypeOwned,
}

#[derive(Debug)]
pub struct TypeInfo<'a> {
    pub name: &'a str,
    pub ty: ComplexType<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum TypeSemantic {
    Val,
    Ref,
}

pub trait Enum {
    const NAME: &'static str;
    const VARIANTS: &'static [&'static str];
}

pub trait Struct {
    const NAME: &'static str;
    const SEMANTIC: TypeSemantic;
    const FIELDS: &'static [TypeInfo<'static>];
}

pub trait Union {
    const NAME: &'static str;
    const VARIANTS: &'static [(&'static str, Option<&'static str>)];
}
