use mech3ax_metadata_types::TypeInfoBase;
use std::borrow::Cow;
use std::collections::HashSet;

/// The C# type kind that is also null-aware.
///
/// Value and reference are supported, while output doesn't make sense.
///
/// # See also
///
/// https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/types/
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// A C# type consisting of a name and kind.
///
/// This is used for type resolution, and should be cheap to construct.
///
/// For generic types, the generic parameters must be part of the name, for
/// example "Foo<T>".
#[derive(Debug, Clone, PartialEq)]
pub struct CSharpType {
    /// The fully-qualified C# type name.
    pub name: Cow<'static, str>,
    pub kind: TypeKind,
    pub generics: Option<HashSet<&'static str>>,
}

impl From<&TypeInfoBase> for CSharpType {
    fn from(base: &TypeInfoBase) -> Self {
        use TypeInfoBase::*;
        let (name, kind) = match base {
            Bool => ("bool", TypeKind::Val),
            U8 => ("byte", TypeKind::Val),
            U16 => ("ushort", TypeKind::Val),
            U32 => ("uint", TypeKind::Val),
            I8 => ("sbyte", TypeKind::Val),
            I16 => ("short", TypeKind::Val),
            I32 => ("int", TypeKind::Val),
            F32 => ("float", TypeKind::Val),
            DateTime => ("System.DateTime", TypeKind::Val),
            String => ("string", TypeKind::Ref),
        };
        let name = Cow::Borrowed(name);
        Self {
            name,
            kind,
            generics: None,
        }
    }
}

impl CSharpType {
    /// Whether the type kind requires a null check when deserializing.
    ///
    /// This is kinda an implementation detail of the C# JSON library, but:
    ///
    /// * Non-null value types won't be deserialized as null
    /// * Nullable values or reference types can be null
    /// * Non-null reference types can be deserialized as null, even though
    ///   this shouldn't be allowed in a null-aware context
    ///
    /// The last case is the one that needs to be handled.
    pub const fn null_check(&self) -> bool {
        matches!(self.kind, TypeKind::Ref)
    }

    pub fn is_byte(&self) -> bool {
        self.name == "byte" && self.kind == TypeKind::Val && self.generics.is_none()
    }

    pub fn byte_vec() -> Self {
        Self {
            name: Cow::Borrowed("byte[]"),
            kind: TypeKind::Val,
            generics: None,
        }
    }

    /// Convert a type into an option/nullable type.
    pub fn option(inner: Self) -> Self {
        let (name, kind) = match inner.kind {
            TypeKind::Val => (Cow::Owned(format!("{}?", inner.name)), TypeKind::ValNull),
            TypeKind::Ref => (Cow::Owned(format!("{}?", inner.name)), TypeKind::RefNull),
            TypeKind::ValNull => {
                eprintln!("WARNING: doubly-nullable value type `{}`", inner.name);
                (inner.name, TypeKind::ValNull)
            }
            TypeKind::RefNull => {
                eprintln!("WARNING: doubly-nullable ref type `{}`", inner.name);
                (inner.name, TypeKind::RefNull)
            }
        };
        Self {
            name,
            kind,
            generics: inner.generics,
        }
    }

    /// Convert a type into a vec/list type.
    pub fn vec(inner: Self) -> Self {
        let name = Cow::Owned(format!("System.Collections.Generic.List<{}>", inner.name));
        // a list is always a reference type, and has it's own nullability,
        // independent of the inner type.
        Self {
            name,
            kind: TypeKind::Ref,
            generics: inner.generics,
        }
    }
}
