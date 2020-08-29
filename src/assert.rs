use std::cmp::{PartialEq, PartialOrd};
use std::fmt::{self, Debug, Display};

pub struct AssertionError(pub String);

impl Debug for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

type Result<T> = ::std::result::Result<T, AssertionError>;

pub fn is_equal_to<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialEq + PartialOrd + Debug,
    U: Display,
{
    if actual == expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected '{}' == {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[allow(dead_code)]
pub fn is_not_equal_to<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialEq + PartialOrd + Debug,
    U: Display,
{
    if actual != expected {
        Ok(())
    } else {
        let msg = format!("Expected '{}' != {:#?}, but was (at {})", name, actual, pos);
        Err(AssertionError(msg))
    }
}

pub fn is_less_than<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialEq + PartialOrd + Debug,
    U: Display,
{
    if actual < expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected '{}' < {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

pub fn is_less_than_or_equal_to<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialEq + PartialOrd + Debug,
    U: Display,
{
    if actual <= expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected '{}' <= {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

pub fn is_greater_than<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialEq + PartialOrd + Debug,
    U: Display,
{
    if actual > expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected '{}' > {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[allow(dead_code)]
pub fn is_greater_than_or_equal_to<S, T, U>(name: S, expected: T, actual: T, pos: U) -> Result<()>
where
    S: Display,
    T: PartialEq + PartialOrd + Debug,
    U: Display,
{
    if actual >= expected {
        Ok(())
    } else {
        let msg = format!(
            "Expected '{}' >= {:#?}, but was {:#?} (at {})",
            name, expected, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[allow(dead_code)]
pub fn is_between<S, T, U>(
    name: S,
    expected_min: T,
    expected_max: T,
    actual: T,
    pos: U,
) -> Result<()>
where
    S: Display,
    T: PartialEq + PartialOrd + Debug,
    U: Display,
{
    if expected_min <= actual && actual <= expected_max {
        Ok(())
    } else {
        let msg = format!(
            "Expected {:#?} <= '{}' <= {:#?}, but was {:#?} (at {})",
            expected_min, name, expected_max, actual, pos
        );
        Err(AssertionError(msg))
    }
}

#[allow(dead_code)]
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
            "Expected '{}' to be in {:#?}, but was {:#?} (at {})",
            name, haystack, needle, pos
        );
        Err(AssertionError(msg))
    }
}

pub fn assert_utf8<S, U, F, T>(name: S, pos: U, func: F) -> Result<T>
where
    S: Display,
    U: Display,
    F: FnOnce() -> ::std::result::Result<T, ::std::str::Utf8Error>,
{
    func().map_err(|_| {
        let msg = format!("Expected '{}' to be a valid string (at {})", name, pos);
        AssertionError(msg)
    })
}

#[allow(dead_code)]
pub fn assert_all_zero<S, U>(name: S, pos: U, buf: &[u8]) -> Result<()>
where
    S: Display,
    U: Display,
{
    let mut iter = buf.iter().enumerate();

    if !iter.all(|(_, &v)| v == 0) {
        let index = iter.next().map(|(i, _)| i).unwrap_or(buf.len() - 1);
        let value = buf[index];
        let msg = format!(
            "Expected '{}' to be zero, but byte {} was {:02X} (at {})",
            name, index, value, pos
        );
        Err(AssertionError(msg))
    } else {
        Ok(())
    }
}

#[macro_export]
macro_rules! assert_that {
    ($name:expr, $actual:tt == $expected:expr, $pos:expr) => {
        $crate::assert::is_equal_to($name, $expected, $actual, $pos)
    };
    ($name:expr, $actual_struct:ident.$actual_field:tt == $expected:expr, $pos:expr) => {
        $crate::assert::is_equal_to($name, $expected, $actual_struct.$actual_field, $pos)
    };
    ($name:expr, $actual:tt != $expected:expr , $pos:expr) => {
        $crate::assert::is_not_equal_to($name, $expected, $actual, $pos)
    };
    ($name:expr, $actual_struct:ident.$actual_field:tt != $expected:expr, $pos:expr) => {
        $crate::assert::is_not_equal_to($name, $expected, $actual_struct.$actual_field, $pos)
    };
    ($name:expr, $actual:tt < $expected:expr, $pos:expr) => {
        $crate::assert::is_less_than($name, $expected, $actual, $pos)
    };
    ($name:expr, $actual_struct:ident.$actual_field:tt < $expected:expr, $pos:expr) => {
        $crate::assert::is_less_than($name, $expected, $actual_struct.$actual_field, $pos)
    };
    // this expected has to be tt, otherwise between won't work
    ($name:expr, $actual:tt <= $expected:tt, $pos:expr) => {
        $crate::assert::is_less_than_or_equal_to($name, $expected, $actual, $pos)
    };
    ($name:expr, $actual_struct:ident.$actual_field:tt <= $expected:expr, $pos:expr) => {
        $crate::assert::is_less_than_or_equal_to(
            $name,
            $expected,
            $actual_struct.$actual_field,
            $pos,
        )
    };
    ($name:expr, $actual:tt > $expected:expr, $pos:expr) => {
        $crate::assert::is_greater_than($name, $expected, $actual, $pos)
    };
    ($name:expr, $actual_struct:ident.$actual_field:tt > $expected:expr, $pos:expr) => {
        $crate::assert::is_greater_than($name, $expected, $actual_struct.$actual_field, $pos)
    };
    ($name:expr, $actual:tt >= $expected:expr, $pos:expr) => {
        $crate::assert::is_greater_than_or_equal_to($name, $expected, $actual, $pos)
    };
    ($name:expr, $actual_struct:ident.$actual_field:tt >= $expected:expr, $pos:expr) => {
        $crate::assert::is_greater_than_or_equal_to(
            $name,
            $expected,
            $actual_struct.$actual_field,
            $pos,
        )
    };
    ($name:expr, $expected_min:tt <= $actual:tt <= $expected_max:expr, $pos:expr) => {
        $crate::assert::is_between($name, $expected_min, $expected_max, $actual, $pos)
    };
    ($name:expr, $expected_min:tt <= $actual_struct:ident.$actual_field:tt <= $expected_max:expr, $pos:expr) => {
        $crate::assert::is_between(
            $name,
            $expected_min,
            $expected_max,
            $actual_struct.$actual_field,
            $pos,
        )
    };
    ($name:expr, -$expected_min:tt <= $actual_struct:ident.$actual_field:tt <= $expected_max:expr, $pos:expr) => {
        $crate::assert::is_between(
            $name,
            -$expected_min,
            $expected_max,
            $actual_struct.$actual_field,
            $pos,
        )
    };
    ($name:expr, $actual:tt in $haystack:expr, $pos:expr) => {
        $crate::assert::is_in($name, $haystack, $actual, $pos)
    };
    ($name:expr, $actual_struct:ident.$actual_field:tt in $haystack:expr, $pos:expr) => {
        $crate::assert::is_in($name, $haystack, $actual_struct.$actual_field, $pos)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn is_equal_to() {
        assert_that!("foo", 1 == 1, 0).unwrap();
        let err = assert_that!("foo", 2 == 1, 0).unwrap_err();
        assert_eq!(
            format!("{:#?}", err),
            "Expected 'foo' == 1, but was 2 (at 0)"
        );
    }

    #[test]
    fn is_not_equal_to() {
        assert_that!("foo", 2 != 1, 0).unwrap();
        let err = assert_that!("foo", 1 != 1, 0).unwrap_err();
        assert_eq!(format!("{:#?}", err), "Expected 'foo' != 1, but was (at 0)");
    }

    #[test]
    fn is_less_than() {
        assert_that!("foo", 1 < 2, 0).unwrap();
        let err = assert_that!("foo", 2 < 1, 0).unwrap_err();
        assert_eq!(
            format!("{:#?}", err),
            "Expected 'foo' < 1, but was 2 (at 0)"
        );
    }

    #[test]
    fn is_less_than_or_equal_to() {
        assert_that!("foo", 1 <= 2, 0).unwrap();
        assert_that!("foo", 2 <= 2, 0).unwrap();
        let err = assert_that!("foo", 3 <= 2, 0).unwrap_err();
        assert_eq!(
            format!("{:#?}", err),
            "Expected 'foo' <= 2, but was 3 (at 0)"
        );
    }

    #[test]
    fn is_greater_than() {
        assert_that!("foo", 2 > 1, 0).unwrap();
        let err = assert_that!("foo", 1 > 2, 0).unwrap_err();
        assert_eq!(
            format!("{:#?}", err),
            "Expected 'foo' > 2, but was 1 (at 0)"
        );
    }

    #[test]
    fn is_greater_than_or_equal_to() {
        assert_that!("foo", 3 >= 2, 0).unwrap();
        assert_that!("foo", 2 >= 2, 0).unwrap();
        let err = assert_that!("foo", 1 >= 2, 0).unwrap_err();
        assert_eq!(
            format!("{:#?}", err),
            "Expected 'foo' >= 2, but was 1 (at 0)"
        );
    }

    #[test]
    fn is_between() {
        assert_that!("foo", 1 <= 1 <= 2, 0).unwrap();
        assert_that!("foo", 1 <= 2 <= 2, 0).unwrap();
        let err = assert_that!("foo", 1 <= 3 <= 2, 0).unwrap_err();
        assert_eq!(
            format!("{:#?}", err),
            "Expected 1 <= 'foo' <= 2, but was 3 (at 0)"
        );
    }
}
