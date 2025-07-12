use mech3ax_types::Maybe;
use mech3ax_types::maybe::{PrimitiveRepr, SupportsMaybe};
use std::cmp::{PartialEq, PartialOrd};
use std::fmt;

#[derive(Clone)]
pub struct CheckError {
    msg: Box<str>,
    file: &'static str,
    line: u32,
}

impl CheckError {
    pub fn new(msg: String, file: &'static str, line: u32) -> Self {
        Self {
            msg: msg.into_boxed_str(),
            file,
            line,
        }
    }
}

impl fmt::Debug for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n({}: {})", self.msg, self.file, self.line)
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n({}: {})", self.msg, self.file, self.line)
    }
}

impl std::error::Error for CheckError {}

type Result<T> = std::result::Result<T, String>;

#[macro_export]
macro_rules! err {
    ($fmt:literal) => {{
        const MSG: &str = $fmt;
        $crate::check::CheckError::new(MSG.to_string(), file!(), line!()).into()
    }};
    ($fmt:literal, $($arg:tt)+) => {{
        let msg: String = format!($fmt, $($arg)+);
        $crate::check::CheckError::new(msg, file!(), line!()).into()
    }};
    ($msg:ident) => {{
        $crate::check::CheckError::new($msg, file!(), line!())
    }};
}

#[inline]
pub fn length_err(msg: String, name: &str, file: &'static str, line: u32) -> CheckError {
    CheckError::new(format!("Too many {name}: {msg}"), file, line)
}

#[macro_export]
macro_rules! len {
    ($len:expr, $name:literal) => {
        ::mech3ax_api_types::Count::check_usize($len)
            .map_err(|msg| $crate::check::length_err(msg, $name, file!(), line!()))
    };
}

#[inline]
pub fn amend_err(
    msg: String,
    name: &str,
    offset: usize,
    file: &'static str,
    line: u32,
) -> CheckError {
    let msg = format!("Assert failed for `{name}` at {offset}: {msg}",);
    CheckError::new(msg, file, line)
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
    ($offset:expr, $field:ident == $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::eq(&$field, &$expected).map_err(|msg| {
            let name = chk!(@name $field);
            let offset = chk!(@offset $field, $offset);
            $crate::check::amend_err(msg, name, offset, FILE, LINE)
        })
    }};
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

    ($offset:expr, $field:ident != $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::ne(&$field, &$expected).map_err(|msg| {
            let name = chk!(@name $field);
            let offset = chk!(@offset $field, $offset);
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

    ($offset:expr, $field:ident < $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::lt(&$field, &$expected).map_err(|msg| {
            let name = chk!(@name $field);
            let offset = chk!(@offset $field, $offset);
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

    ($offset:expr, $field:ident > $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::gt(&$field, &$expected).map_err(|msg| {
            let name = chk!(@name $field);
            let offset = chk!(@offset $field, $offset);
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

    ($offset:expr, $field:ident <= $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::le(&$field, &$expected).map_err(|msg| {
            let name = chk!(@name $field);
            let offset = chk!(@offset $field, $offset);
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

    ($offset:expr, $field:ident >= $expected:expr) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::ge(&$field, &$expected).map_err(|msg| {
            let name = chk!(@name $field);
            let offset = chk!(@offset $field, $offset);
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

    ($offset:expr, ?$field:ident) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $crate::check::maybe($field).map_err(|msg| {
            let name = chk!(@name $field);
            let offset = chk!(@offset $field, $offset);
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

    ($offset:expr, $func:ident($field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func($field $(, $arg)*).map_err(|msg| {
            let name = chk!(@name $field);
            let offset = chk!(@offset $field, $offset);
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

    ($offset:expr, $func:ident(&$field:ident $(, $arg:expr)* $(,)?)) => {{
        const FILE: &str = file!();
        const LINE: u32 = line!();
        $func(&$field $(, $arg)*).map_err(|msg| {
            let name = chk!(@name $field);
            let offset = chk!(@offset $field, $offset);
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

    // ($offset:expr => $name:literal, $value:ident == $expected:expr) => {{
    //     const FILE: &str = file!();
    //     const LINE: u32 = line!();
    //     $crate::check::eq(&$value, &$expected).map_err(|msg| {
    //         let name: &str = $name;
    //         let offset: usize = $offset;
    //         $crate::check::amend_err(msg, name, offset, FILE, LINE)
    //     })
    // }};

    (@name $field:ident) => { stringify!($field) };
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
    (@offset $field:ident, $offset:expr) => {{
        let base_offset: usize = $offset;
        base_offset
    }};
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
