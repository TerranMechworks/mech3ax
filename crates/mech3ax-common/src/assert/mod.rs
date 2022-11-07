use crate::string::ConversionError;
use num_traits::AsPrimitive;
use std::cmp::{PartialEq, PartialOrd};
use std::fmt::{self, Debug, Display};
use thiserror::Error;

#[derive(Error)]
pub struct AssertionError(pub String);

impl Debug for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

type Result<T> = ::std::result::Result<T, AssertionError>;

#[inline]
pub fn is_equal_to<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialEq + Debug,
    U: Display,
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
pub fn is_not_equal_to<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialEq + Debug,
    U: Display,
{
    if actual != expected {
        Ok(())
    } else {
        let msg = format!("Expected `{}` != {:#?}, but was (at {})", name, actual, pos);
        Err(AssertionError(msg))
    }
}

#[inline]
pub fn is_less_than<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialOrd + Debug,
    U: Display,
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
pub fn is_less_than_or_equal_to<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialOrd + Debug,
    U: Display,
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
pub fn is_greater_than<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialOrd + Debug,
    U: Display,
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
#[allow(dead_code)]
pub fn is_greater_than_or_equal_to<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialOrd + Debug,
    U: Display,
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
pub fn is_between<S, T, U>(
    name: S,
    expected_min: T,
    expected_max: T,
    actual: T,
    pos: U,
) -> Result<()>
where
    S: Display,
    T: PartialOrd + Debug,
    U: Display,
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
pub fn is_in<S, T, U>(name: S, haystack: &[T], needle: &T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialEq + Debug,
    U: Display,
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
pub fn is_bool<S, U>(name: S, actual: u32, pos: U) -> Result<bool>
where
    S: Display,
    U: Display,
{
    if actual > 1 {
        let msg = format!(
            "Expected `{}` to be 0 or 1, but was {} (at {})",
            name, actual, pos
        );
        Err(AssertionError(msg))
    } else {
        Ok(actual == 1)
    }
}

#[inline]
pub fn assert_utf8<S, U, F, T>(name: S, pos: U, func: F) -> Result<T>
where
    S: Display,
    U: AsPrimitive<usize>,
    F: FnOnce() -> ::std::result::Result<T, ConversionError>,
{
    func().map_err(|err| {
        let pos = pos.as_();
        let msg = match err {
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
    })
}

#[inline]
pub fn assert_all_zero<S, U>(name: S, pos: U, buf: &[u8]) -> Result<()>
where
    S: Display,
    U: Display,
{
    let mut iter = buf.iter().enumerate();

    if let Some((index, _)) = iter.find(|(_, &v)| v != 0) {
        let value = buf[index];
        let msg = format!(
            "Expected `{}` to be zero, but byte {} was {:02X} (at {})",
            name, index, value, pos
        );
        Err(AssertionError(msg))
    } else {
        Ok(())
    }
}

#[macro_export]
macro_rules! assert_that {
    ($name:expr, $expected_min:tt <= $($actual:tt).+ <= $expected_max:expr, $pos:expr) => {
        $crate::assert::is_between($name, $expected_min, $expected_max, $($actual).+, $pos)
    };
    ($name:expr, -$expected_min:tt <= $($actual:tt).+ <= $expected_max:expr, $pos:expr) => {
        $crate::assert::is_between($name, -$expected_min, $expected_max, $($actual).+, $pos)
    };
    ($name:expr, $($actual:tt).+ == $expected:expr, $pos:expr) => {
        $crate::assert::is_equal_to($name, $expected, $($actual).+, $pos)
    };
    ($name:expr, &$($actual:tt).+ == $expected:expr, $pos:expr) => {
        $crate::assert::is_equal_to($name, $expected, &$($actual).+, $pos)
    };
    ($name:expr, $($actual:tt).+ != $expected:expr, $pos:expr) => {
        $crate::assert::is_not_equal_to($name, $expected, $($actual).+, $pos)
    };
    ($name:expr, &$($actual:tt).+ != $expected:expr, $pos:expr) => {
        $crate::assert::is_not_equal_to($name, $expected, &$($actual).+, $pos)
    };
    ($name:expr, $($actual:tt).+ < $expected:expr, $pos:expr) => {
        $crate::assert::is_less_than($name, $expected, $($actual).+, $pos)
    };
    ($name:expr, $($actual:tt).+ <= $expected:expr, $pos:expr) => {
        $crate::assert::is_less_than_or_equal_to($name, $expected, $($actual).+, $pos)
    };
    ($name:expr, $($actual:tt).+ > $expected:expr, $pos:expr) => {
        $crate::assert::is_greater_than($name, $expected, $($actual).+, $pos)
    };
    ($name:expr, $($actual:tt).+ >= $expected:expr, $pos:expr) => {
        $crate::assert::is_greater_than_or_equal_to( $name, $expected, $($actual).+, $pos)
    };
    ($name:expr, $($actual:tt).+ in $haystack:expr, $pos:expr) => {
        $crate::assert::is_in($name, &$haystack, &$($actual).+, $pos)
    };
    ($name:expr, bool $actual:expr, $pos:expr) => {
        $crate::assert::is_bool($name, $actual, $pos)
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
macro_rules! bool_c {
    ($value:expr) => {
        if $value {
            1
        } else {
            0
        }
    };
}

#[macro_export]
macro_rules! assert_len {
    ($ty:ty, $value:expr, $name:literal) => {{
        let value = $value;
        let r: $crate::Result<$ty> = value.try_into().map_err(|_e| {
            $crate::assert_with_msg!(
                "Too big: `{}` must be <= {}, but was {}",
                $name,
                <$ty>::MAX,
                value,
            )
        });
        r
    }};
}

#[cfg(test)]
mod tests;
