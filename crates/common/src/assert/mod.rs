use mech3ax_types::maybe::SupportsMaybe;
use mech3ax_types::{Bitflags, Bool, ConversionError, Maybe, PrimitiveEnum, PrimitiveRepr};
use std::cmp::{PartialEq, PartialOrd};
use std::fmt;

#[derive(Clone)]
pub struct AssertionError(pub String);

impl fmt::Debug for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AssertionError {}

type Result<T> = ::std::result::Result<T, AssertionError>;

#[inline]
pub fn is_equal_to<T, U>(name: &str, pos: usize, expected: U, actual: T) -> Result<()>
where
    T: PartialEq<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual == expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected `{}` == {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn is_not_equal_to<T, U>(name: &str, pos: usize, expected: U, actual: T) -> Result<()>
where
    T: PartialEq<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual != expected {
        Ok(())
    } else {
        let msg = format!("Expected `{}` != {:#?}, but was (at {})", name, actual, pos);
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn is_less_than<T, U>(name: &str, pos: usize, expected: U, actual: T) -> Result<()>
where
    T: PartialOrd<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual < expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected `{}` < {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn is_less_than_or_equal_to<T, U>(name: &str, pos: usize, expected: U, actual: T) -> Result<()>
where
    T: PartialOrd<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual <= expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected `{}` <= {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn is_greater_than<T, U>(name: &str, pos: usize, expected: U, actual: T) -> Result<()>
where
    T: PartialOrd<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual > expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected `{}` > {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn is_greater_than_or_equal_to<T, U>(
    name: &str,
    pos: usize,
    expected: U,
    actual: T,
) -> Result<()>
where
    T: PartialOrd<U> + fmt::Debug,
    U: fmt::Debug,
{
    if actual >= expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected `{}` >= {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn is_between<T>(
    name: &str,
    pos: usize,
    expected_min: T,
    expected_max: T,
    actual: T,
) -> Result<()>
where
    T: PartialOrd + fmt::Debug,
{
    if expected_min <= actual && actual <= expected_max {
        Ok(())
    } else {
        let msg = format!(
            "Expected {:#?} <= `{}` <= {:#?}, but was {:#?} (at {})",
            expected_min, name, expected_max, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn is_in_slice<T>(name: &str, pos: usize, haystack: &[T], needle: &T) -> Result<()>
where
    T: PartialEq + fmt::Debug,
{
    if haystack.contains(needle) {
        Ok(())
    } else {
        let msg = format!(
            "Expected `{}` to be in {:#?}, but was {:#?} (at {})",
            name, haystack, needle, pos
        );
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn is_in_range<T>(name: &str, pos: usize, start: T, end: T, needle: T) -> Result<()>
where
    T: Copy + PartialOrd + fmt::Display,
{
    if needle >= start && needle <= end {
        Ok(())
    } else {
        let msg = format!(
            "Expected `{}` to be in {}..{}, but was {} (at {})",
            name, start, end, needle, pos
        );
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn format_conversion_err(name: &str, pos: usize, e: ConversionError) -> AssertionError {
    let msg = match e {
        ConversionError::PaddingError(padding) => format!(
            "Expected `{}` to padded with {} (at {})",
            name, padding, pos
        ),
        ConversionError::NonAscii(index) => {
            format!(
                "Expected `{}` to be a valid string (at {})",
                name,
                pos + index
            )
        }
        ConversionError::Unterminated => {
            format!("Expected `{}` to be zero-terminated (at {})", name, pos)
        }
    };
    AssertionError(msg)
}

#[inline]
pub fn assert_utf8<F, T>(name: &str, pos: usize, func: F) -> Result<T>
where
    F: FnOnce() -> ::std::result::Result<T, ConversionError>,
{
    func().map_err(|e| format_conversion_err(name, pos, e))
}

#[inline]
pub fn is_all_zero(name: &str, pos: usize, buf: &[u8]) -> Result<()> {
    let mut iter = buf.iter().copied();
    if let Some(index) = iter.position(|v| v != 0) {
        let value = buf[index];
        let msg = format!(
            "Expected `{}` to be zero, but byte {} was {:02X} (at {})",
            name,
            index,
            value,
            pos + index
        );
        Err(AssertionError(msg))
    } else {
        Ok(())
    }
}

#[inline]
pub fn is_bool<R>(name: &str, pos: usize, v: Bool<R>) -> Result<bool>
where
    R: PrimitiveRepr,
    bool: SupportsMaybe<R>,
{
    v.validate().ok_or_else(|| {
        let msg = format!(
            "Expected `{}` to be 0 or 1, but was {} (at {})",
            name, v, pos
        );
        AssertionError(msg)
    })
}

#[inline]
pub fn is_bitflags<R, F>(name: &str, pos: usize, v: Maybe<R, F>) -> Result<F>
where
    R: PrimitiveRepr,
    F: Bitflags<R>,
{
    v.validate().ok_or_else(|| {
        let msg = format!(
            "Expected `{}` to have valid flags, but was {} (at {})",
            name, v, pos
        );
        AssertionError(msg)
    })
}

#[inline]
pub fn is_enum<R, E>(name: &str, pos: usize, v: Maybe<R, E>) -> Result<E>
where
    R: PrimitiveRepr,
    E: PrimitiveEnum<R>,
{
    v.validate().ok_or_else(|| {
        let discriminants = mech3ax_types::primitive_enum::format_discriminants(E::DISCRIMINANTS);
        let msg = format!(
            "Expected `{}` to be {}, but was {} (at {})",
            name, discriminants, v, pos
        );
        AssertionError(msg)
    })
}

fn struct_cmp<S>(name: &str, pos: usize, expected: &S, actual: &S) -> AssertionError
where
    S: mech3ax_types::AsBytes + Default,
{
    let expected = expected.as_bytes();
    let actual = actual.as_bytes();
    assert_eq!(expected.len(), actual.len(), "struct bytes len");
    let rel_pos = expected
        .iter()
        .copied()
        .zip(actual.iter().copied())
        .position(|(a, b)| a != b)
        .unwrap_or(0);
    let abs_pos = pos + rel_pos;

    let type_name = std::any::type_name::<S>();
    AssertionError(format!(
        "Expected `{}` to be `{}::default()`, mismatch position {}/{} (at {})",
        name, type_name, rel_pos, abs_pos, pos,
    ))
}

#[inline]
pub fn is_default<S>(name: &str, pos: usize, v: &S) -> Result<()>
where
    S: mech3ax_types::AsBytes + Default + PartialEq,
{
    let default = S::default();
    if v == &default {
        Ok(())
    } else {
        Err(struct_cmp(name, pos, &default, v))
    }
}

#[macro_export]
macro_rules! assert_that {
    ($name:expr, $expected_min:tt <= $($actual:tt).+ <= $expected_max:expr, $pos:expr) => {
        $crate::assert::is_between($name, $pos, &$expected_min, &$expected_max, &$($actual).+)
    };
    ($name:expr, -$expected_min:tt <= $($actual:tt).+ <= $expected_max:expr, $pos:expr) => {
        $crate::assert::is_between($name, $pos, &-$expected_min, &$expected_max, &$($actual).+)
    };
    ($name:expr, $($actual:tt).+ == $expected:expr, $pos:expr) => {
        $crate::assert::is_equal_to($name, $pos, &$expected, &$($actual).+)
    };
    ($name:expr, $($actual:tt).+ eq $expected:expr, $pos:expr) => {
        $crate::assert::is_equal_to($name, $pos, $expected, $($actual).+.as_str())
    };
    ($name:expr, $($actual:tt).+ != $expected:expr, $pos:expr) => {
        $crate::assert::is_not_equal_to($name, $pos, &$expected, &$($actual).+)
    };
    ($name:expr, &$($actual:tt).+ != $expected:expr, $pos:expr) => {
        $crate::assert::is_not_equal_to($name, $pos, &$expected, &$($actual).+)
    };
    ($name:expr, $($actual:tt).+ < $expected:expr, $pos:expr) => {
        $crate::assert::is_less_than($name, $pos, &$expected, &$($actual).+)
    };
    ($name:expr, $($actual:tt).+ <= $expected:expr, $pos:expr) => {
        $crate::assert::is_less_than_or_equal_to($name, $pos, &$expected, &$($actual).+)
    };
    ($name:expr, $($actual:tt).+ > $expected:expr, $pos:expr) => {
        $crate::assert::is_greater_than($name, $pos, &$expected, &$($actual).+)
    };
    ($name:expr, $($actual:tt).+ >= $expected:expr, $pos:expr) => {
        $crate::assert::is_greater_than_or_equal_to($name, $pos, &$expected, &$($actual).+)
    };
    ($name:expr, $($actual:tt).+ in ($start:literal..$end:literal), $pos:expr) => {
        $crate::assert::is_in_range($name, $pos, $start, $end, $($actual).+)
    };
    ($name:expr, $($actual:tt).+ in $haystack:expr, $pos:expr) => {
        $crate::assert::is_in_slice($name, $pos, &$haystack, &$($actual).+)
    };
    ($name:expr, bool $actual:expr, $pos:expr) => {
        $crate::assert::is_bool($name, $pos, $actual)
    };
    ($name:expr, enum $actual:expr, $pos:expr) => {
        $crate::assert::is_enum($name, $pos, $actual)
    };
    ($name:expr, zero $actual:expr, $pos:expr) => {
        $crate::assert::is_all_zero($name, $pos, &$actual)
    };
    ($name:expr, flags $actual:expr, $pos:expr) => {
        $crate::assert::is_bitflags($name, $pos, $actual)
    };
    ($name:expr, default $actual:expr, $pos:expr) => {
        $crate::assert::is_default($name, $pos, &$actual)
    };
}

#[macro_export]
macro_rules! assert_with_msg {
    ($msg:literal) => {
        $crate::assert_with_msg!($msg.to_string())
    };
    ($msg:expr) => {
        $crate::Error::Assert(
            $crate::assert::AssertionError($msg)
        )
    };
    ($($arg:tt)*) => {
        $crate::assert_with_msg!(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! assert_len {
    ($ty:ty, $value:expr, $name:literal) => {{
        let value: usize = $value;
        <$ty>::try_from(value).map_err(|_e| {
            $crate::Error::Assert($crate::assert::AssertionError(format!(
                "`{}` must be <= {max}, but was {value}",
                $name,
                max = <$ty>::MAX,
                value = value,
            )))
        })
    }};
    ($ty:ty, $value:expr, $name:literal, $(arg:expr),+ $(,)?) => {{
        let value: usize = $value;
        <$ty>::try_from(value).map_err(|_e| {
            $crate::Error::Assert($crate::assert::AssertionError(format!(
                concat!("`", $name, "` must be <= {max}, but was {value}"),
                $name,
                $($arg,)+
                max = <$ty>::MAX,
                value = value,
            )))
        })
    }};
}

#[cfg(test)]
mod tests;
