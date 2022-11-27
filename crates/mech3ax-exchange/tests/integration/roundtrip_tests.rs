use super::Bytes;
use mech3ax_exchange::{from_slice, to_vec};

macro_rules! round_trip {
    ($type:ty, $value:expr) => {
        let expected: $type = $value;
        let bin = to_vec(&expected).unwrap();
        let actual: $type = from_slice(&bin).unwrap();
        assert_eq!(actual, expected);
    };
}

#[test]
fn i8_tests() {
    round_trip!(i8, 0);
    round_trip!(i8, 1);
    round_trip!(i8, -1);
    round_trip!(i8, i8::MIN);
    round_trip!(i8, i8::MAX);
}

#[test]
fn i16_tests() {
    round_trip!(i16, 0);
    round_trip!(i16, 1);
    round_trip!(i16, -1);
    round_trip!(i16, i16::MIN);
    round_trip!(i16, i16::MAX);
}

#[test]
fn i32_tests() {
    round_trip!(i32, 0);
    round_trip!(i32, 1);
    round_trip!(i32, -1);
    round_trip!(i32, i32::MIN);
    round_trip!(i32, i32::MAX);
}

#[test]
fn u8_tests() {
    round_trip!(u8, u8::MIN);
    round_trip!(u8, 1);
    round_trip!(u8, u8::MAX);
}

#[test]
fn u16_tests() {
    round_trip!(u16, u16::MIN);
    round_trip!(u16, 1);
    round_trip!(u16, u16::MAX);
}

#[test]
fn u32_tests() {
    round_trip!(u32, u32::MIN);
    round_trip!(u32, 1);
    round_trip!(u32, u32::MAX);
}

#[test]
fn f32_tests() {
    round_trip!(f32, 0.0);
    round_trip!(f32, -0.0);
    round_trip!(f32, 1.0);
    round_trip!(f32, -1.0);
    round_trip!(f32, f32::MIN);
    round_trip!(f32, f32::MAX);
}

#[test]
fn bool_tests() {
    round_trip!(bool, false);
    round_trip!(bool, true);
}

#[test]
fn string_tests() {
    round_trip!(String, String::from("foo"));
    // deserializing a borrowed &str is not supported
}

#[test]
fn bytes_tests() {
    round_trip!(Bytes, Bytes::from(b"foo"));
    // deserializing a borrowed &[u8] is not supported
}

#[test]
fn option_tests() {
    round_trip!(Option<bool>, None);
    round_trip!(Option<bool>, Some(true));
    round_trip!(Option<bool>, Some(false));
}

#[test]
fn seq_tests() {
    round_trip!(Vec<bool>, vec![]);
    round_trip!(Vec<bool>, vec![false]);
    round_trip!(Vec<bool>, vec![false, true]);
}

#[test]
fn field_struct_tests() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Field {
        a: bool,
    }

    round_trip!(Field, Field { a: false });
    round_trip!(Field, Field { a: true });
}

#[test]
fn field_struct_with_optionals_tests() {
    let bin = {
        #[derive(Debug, Clone, PartialEq, serde::Serialize)]
        struct Optional {
            // definitely doesn't have a
            b: bool,
        }

        to_vec(&Optional { b: true }).unwrap()
    };

    #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
    struct Optional {
        #[serde(default)]
        a: bool,
        b: bool,
    }

    let actual: Optional = from_slice(&bin).unwrap();
    let expected = Optional { a: false, b: true };
    assert_eq!(actual, expected);
}

#[test]
fn enum_unit_tests() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    enum Unit {
        A,
        B,
    }

    round_trip!(Unit, Unit::A);
    round_trip!(Unit, Unit::B);
}

#[test]
fn enum_newtype_tests() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    enum NewType {
        A(bool),
        B(bool),
    }

    round_trip!(NewType, NewType::A(true));
    round_trip!(NewType, NewType::B(false));
}

#[test]
fn enum_mixed_tests() {
    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    enum Mixed {
        A(bool),
        B,
    }

    round_trip!(Mixed, Mixed::A(true));
    round_trip!(Mixed, Mixed::B);
}
