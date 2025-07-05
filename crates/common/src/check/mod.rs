pub use mech3ax_types::cstruct::CStruct;
use mech3ax_types::maybe::{PrimitiveRepr, SupportsMaybe};
use mech3ax_types::{Bitflags, Bool, Maybe, Padded, PrimitiveEnum};
use std::cmp::{PartialEq, PartialOrd};
use std::fmt;

type Result<T> = std::result::Result<T, String>;

#[macro_export]
macro_rules! err {
    // ($offset:expr, $fmt:literal) => {{
    //     let offset: usize = $offset;
    //     let msg = format!(concat!("Error at {offset}: ", $fmt), offset=offset);
    //     $crate::assert::AssertionError(msg).into()
    // }};
    // ($offset:expr, $fmt:literal, $($arg:tt)+) => {{
    //     let offset: usize = $offset;
    //     let msg = format!(concat!("Error at {offset}: ", $fmt), $($arg)+, offset=offset);
    //     $crate::assert::AssertionError(msg).into()
    // }};
    ($fmt:literal) => {{
        const MSG: &str = concat!($fmt, "\n(", file!(), ":", stringify!(line!()), ")");
        $crate::assert::AssertionError(MSG.to_string()).into()
    }};
    ($fmt:literal, $($arg:tt)+) => {{
        let msg = format!(concat!($fmt, "\n(", file!(), ":", stringify!(line!()), ")"), $($arg)+);
        $crate::assert::AssertionError(msg).into()
    }};
}

#[inline]
pub fn amend_err(
    msg: String,
    name: &str,
    offset: usize,
    file: &str,
    line: u32,
) -> crate::assert::AssertionError {
    // let backtrace = std::backtrace::Backtrace::force_capture();
    crate::assert::AssertionError(format!(
        "Assert failed for `{name}` at {offset}: {msg}\n({file}:{line})",
    ))
}

#[inline]
pub fn eq<T, U>(actual: T, expected: U) -> Result<()>
where
    T: PartialEq<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual == expected {
        Ok(())
    } else {
        Err(format!("expected {:#?} == {:#?}", actual, expected))
    }
}

#[inline]
pub fn ne<T, U>(actual: T, expected: U) -> Result<()>
where
    T: PartialEq<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual != expected {
        Ok(())
    } else {
        Err(format!("expected {:#?} != {:#?}", actual, expected))
    }
}

#[inline]
pub fn lt<T, U>(actual: T, expected: U) -> Result<()>
where
    T: PartialOrd<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual < expected {
        Ok(())
    } else {
        Err(format!("expected {:#?} < {:#?}", actual, expected))
    }
}

#[inline]
pub fn le<T, U>(actual: T, expected: U) -> Result<()>
where
    T: PartialOrd<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual <= expected {
        Ok(())
    } else {
        Err(format!("expected {:#?} <= {:#?}", actual, expected))
    }
}

#[inline]
pub fn gt<T, U>(actual: T, expected: U) -> Result<()>
where
    T: PartialOrd<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual > expected {
        Ok(())
    } else {
        Err(format!("expected {:#?} > {:#?}", actual, expected))
    }
}

#[inline]
pub fn ge<T, U>(actual: T, expected: U) -> Result<()>
where
    T: PartialOrd<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual >= expected {
        Ok(())
    } else {
        Err(format!("expected {:#?} >= {:#?}", actual, expected))
    }
}

#[inline]
pub fn is_bool<R>(v: Bool<R>) -> Result<bool>
where
    R: PrimitiveRepr,
    bool: SupportsMaybe<R>,
{
    v.validate()
        .ok_or_else(|| format!("invalid bool {}, expected 0 or 1", v))
}

#[inline]
pub fn is_enum<R, E>(v: Maybe<R, E>) -> Result<E>
where
    R: PrimitiveRepr,
    E: PrimitiveEnum<R>,
{
    v.validate().ok_or_else(|| {
        let discriminants = mech3ax_types::primitive_enum::format_discriminants(E::DISCRIMINANTS);
        format!("invalid enum {}, expected {}", v, discriminants)
    })
}

#[inline]
pub fn flags<R, F>(v: Maybe<R, F>) -> Result<F>
where
    R: PrimitiveRepr,
    F: Bitflags<R>,
{
    v.validate().ok_or_else(|| format!("invalid flags {}", v))
}

#[inline]
pub fn padded<R, P>(v: Maybe<R, P>) -> Result<P>
where
    R: PrimitiveRepr,
    P: Padded<R>,
{
    v.validate()
        .ok_or_else(|| format!("invalid padding {}, expected {}", v, P::PATTERN))
}

#[macro_export]
macro_rules! chk {
    ($offset:expr, $struct:ident.$field:ident == $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::eq(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$substruct:ident.$field:ident == $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::eq(&$struct.$substruct.$field, &$expected).map_err(|msg| {
            let name = chk!(__name $struct.$substruct.$field);
            let offset = chk!(__offset $struct.$substruct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$field:ident != $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::ne(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$substruct:ident.$field:ident != $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::ne(&$struct.$substruct.$field, &$expected).map_err(|msg| {
            let name = chk!(__name $struct.$substruct.$field);
            let offset = chk!(__offset $struct.$substruct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$field:ident < $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::lt(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$field:ident > $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::gt(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$field:ident >= $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::ge(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, bool $struct:ident.$field:ident) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::is_bool($struct.$field).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, enum $struct:ident.$field:ident) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::is_enum($struct.$field).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, flags $struct:ident.$field:ident) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::flags($struct.$field).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $func:ident($struct:ident.$field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func($struct.$field $(, $arg)*).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $func:ident($struct:ident.$substruct:ident.$field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func($struct.$substruct.$field $(, $arg)*).map_err(|msg| {
            let name = chk!(__name $struct.$substruct.$field);
            let offset = chk!(__offset $struct.$substruct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $func:ident(&$struct:ident.$field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func(&$struct.$field $(, $arg)*).map_err(|msg| {
            let name = chk!(__name $struct.$field);
            let offset = chk!(__offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $func:ident(&$struct:ident.$substruct:ident.$field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func(&$struct.$substruct.$field $(, $arg)*).map_err(|msg| {
            let name = chk!(__name $struct.$substruct.$field);
            let offset = chk!(__offset $struct.$substruct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    (__name $struct:ident.$field:ident) => {
        concat!(stringify!($struct), ".", stringify!($field))
    };
    (__name $struct:ident.$substruct:ident.$field:ident) => {
        concat!(
            stringify!($struct),
            ".",
            stringify!($substruct),
            ".",
            stringify!($field)
        )
    };
    (__offset $struct:ident.$field:ident, $offset:expr) => {{
        let field_offset: usize = $crate::check::CStruct::__field_offsets(&$struct).$field;
        let base_offset: usize = $offset;
        base_offset.wrapping_add(field_offset)
    }};
    (__offset $struct:ident.$substruct:ident.$field:ident, $offset:expr) => {{
        let base_offset: usize = $offset;
        let sub_offset: usize = $crate::check::CStruct::__field_offsets(&$struct).$substruct;
        let field_offset: usize =
            $crate::check::CStruct::__field_offsets(&$struct.$substruct).$field;
        base_offset
            .wrapping_add(sub_offset)
            .wrapping_add(field_offset)
    }};
}

#[cfg(test)]
mod tests;
