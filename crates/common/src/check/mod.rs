use mech3ax_types::maybe::{PrimitiveRepr, SupportsMaybe};
use mech3ax_types::Maybe;
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
pub fn maybe<R: PrimitiveRepr, T: SupportsMaybe<R>>(v: Maybe<R, T>) -> Result<T> {
    v.check()
}

#[inline]
pub const fn __name<T: mech3ax_types::cstruct::CStruct>(_: &T) -> &'static str {
    T::__NAME
}

#[inline]
pub const fn __field_offsets<T: mech3ax_types::cstruct::CStruct>(
    _: &T,
) -> &'static T::FieldOffsets {
    T::__FIELD_OFFSETS
}

#[macro_export]
macro_rules! chk {
    ($offset:expr, $struct:ident.$field:ident == $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::eq(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(@name $struct.$field);
            let offset = chk!(@offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$substruct:ident.$field:ident == $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::eq(&$struct.$substruct.$field, &$expected).map_err(|msg| {
            let name = chk!(@name $struct.$substruct.$field);
            let offset = chk!(@offset $struct.$substruct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$field:ident != $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::ne(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(@name $struct.$field);
            let offset = chk!(@offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$substruct:ident.$field:ident != $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::ne(&$struct.$substruct.$field, &$expected).map_err(|msg| {
            let name = chk!(@name $struct.$substruct.$field);
            let offset = chk!(@offset $struct.$substruct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$field:ident < $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::lt(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(@name $struct.$field);
            let offset = chk!(@offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$field:ident > $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::gt(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(@name $struct.$field);
            let offset = chk!(@offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$field:ident <= $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::le(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(@name $struct.$field);
            let offset = chk!(@offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $struct:ident.$field:ident >= $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::ge(&$struct.$field, &$expected).map_err(|msg| {
            let name = chk!(@name $struct.$field);
            let offset = chk!(@offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, ?$struct:ident.$field:ident) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::maybe($struct.$field).map_err(|msg| {
            let name = chk!(@name $struct.$field);
            let offset = chk!(@offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $func:ident($struct:ident.$field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func($struct.$field $(, $arg)*).map_err(|msg| {
            let name = chk!(@name $struct.$field);
            let offset = chk!(@offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $func:ident($struct:ident.$substruct:ident.$field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func($struct.$substruct.$field $(, $arg)*).map_err(|msg| {
            let name = chk!(@name $struct.$substruct.$field);
            let offset = chk!(@offset $struct.$substruct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $func:ident(&$struct:ident.$field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func(&$struct.$field $(, $arg)*).map_err(|msg| {
            let name = chk!(@name $struct.$field);
            let offset = chk!(@offset $struct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr, $func:ident(&$struct:ident.$substruct:ident.$field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func(&$struct.$substruct.$field $(, $arg)*).map_err(|msg| {
            let name = chk!(@name $struct.$substruct.$field);
            let offset = chk!(@offset $struct.$substruct.$field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    ($offset:expr => $name:literal, $value:ident == $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::eq(&$value, &$expected).map_err(|msg| {
            let name: &str = $name;
            let offset: usize = $offset;
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
    (@name $struct:ident.$field:ident) => {
        concat!(stringify!($struct), ".", stringify!($field))
    };
    (@name $struct:ident.$substruct:ident.$field:ident) => {
        concat!(
            stringify!($struct),
            ".",
            stringify!($substruct),
            ".",
            stringify!($field)
        )
    };
    (@offset $struct:ident.$field:ident, $offset:expr) => {{
        let field_offset: usize = $crate::check::__field_offsets(&$struct).$field;
        let base_offset: usize = $offset;
        base_offset.wrapping_add(field_offset)
    }};
    (@offset $struct:ident.$substruct:ident.$field:ident, $offset:expr) => {{
        let base_offset: usize = $offset;
        let sub_offset: usize = $crate::check::__field_offsets(&$struct).$substruct;
        let field_offset: usize = $crate::check::__field_offsets(&$struct.$substruct).$field;
        base_offset
            .wrapping_add(sub_offset)
            .wrapping_add(field_offset)
    }};
}

#[cfg(test)]
mod tests;
