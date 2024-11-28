use super::{assert_matches, Bytes};
use mech3ax_exchange::{to_vec, ErrorCode};
use std::collections::HashMap;

macro_rules! assert_err {
    ($ty:ty, $v:expr, $code:pat) => {{
        let v: $ty = $v;
        let err = to_vec(&v).unwrap_err();
        assert_matches!(err.code(), $code);
    }};
}

macro_rules! assert_unsupported {
    ($ty:ty, $v:expr) => {
        assert_err!($ty, $v, ErrorCode::UnsupportedType);
    };
}

macro_rules! assert_supported {
    ($ty:ty, $v:expr) => {{
        let v: $ty = $v;
        let _ = to_vec(&v).unwrap();
    }};
}

#[test]
fn signed_tests() {
    assert_supported!(i8, 0);
    assert_supported!(i16, 0);
    assert_supported!(i32, 0);
    assert_unsupported!(i64, 0);
}

#[test]
fn unsigned_tests() {
    assert_supported!(u8, 0);
    assert_supported!(u16, 0);
    assert_supported!(u32, 0);
    assert_supported!(u64, 0);
}

#[test]
fn float_tests() {
    assert_supported!(f32, 0.0);
    assert_unsupported!(f64, 0.0);
}

#[test]
fn bool_tests() {
    assert_supported!(bool, true);
    assert_supported!(bool, false);
}

#[test]
fn char_tests() {
    assert_unsupported!(char, ' ');
}

#[test]
fn string_tests() {
    assert_supported!(String, String::from(""));
    assert_supported!(String, String::from("foo"));
    // we really only expect to serialise owned values, but this also works...
    assert_supported!(&str, "");
    assert_supported!(&str, "foo");
}

#[test]
fn bytes_tests() {
    assert_supported!(Bytes, Bytes::from(b""));
    assert_supported!(Bytes, Bytes::from(b"foo"));
    // we really only expect to serialise owned values, but this also works...
    assert_supported!(bytes, bytes(b""));
    assert_supported!(bytes, bytes(b"foo"));

    // normal byte arrays don't work: https://github.com/serde-rs/serde/issues/518
    // this will instead be serialized as a sequence.
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    struct bytes<'a>(&'a [u8]);

    impl<'a> serde::ser::Serialize for bytes<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::ser::Serializer,
        {
            serializer.serialize_bytes(self.0)
        }
    }
}

#[test]
fn seq_tests() {
    assert_supported!(Vec<bool>, vec![true, false]);
}

#[test]
fn tuple_tests() {
    assert_unsupported!((bool, bool), (false, true));
}

#[test]
fn map_tests() {
    assert_unsupported!(HashMap<bool, bool>, HashMap::new());
}

#[test]
fn unit_tests() {
    assert_unsupported!((), ());
}

#[test]
fn unit_struct_tests() {
    #[derive(serde::Serialize)]
    struct Unit;

    assert_unsupported!(Unit, Unit);
}

#[test]
fn newtype_struct_tests() {
    #[derive(serde::Serialize)]
    struct NewType(bool);

    assert_supported!(NewType, NewType(true));
}

#[test]
fn tuple_struct_tests() {
    #[derive(serde::Serialize)]
    struct Tuple(bool, bool);

    assert_unsupported!(Tuple, Tuple(false, true));
}

#[test]
fn field_struct_tests() {
    #[derive(serde::Serialize)]
    struct Field {
        a: bool,
    }

    assert_supported!(Field, Field { a: false });
}

#[test]
fn enum_unit_tests() {
    #[derive(serde::Serialize)]
    enum Unit {
        A,
    }

    assert_supported!(Unit, Unit::A);
}

#[test]
fn enum_newtype_tests() {
    #[derive(serde::Serialize)]
    enum NewType {
        A(bool),
    }

    assert_supported!(NewType, NewType::A(true));
}

#[test]
fn enum_tuple_tests() {
    #[derive(serde::Serialize)]
    enum Tuple {
        A(bool, bool),
    }

    assert_unsupported!(Tuple, Tuple::A(true, false));
}

#[test]
fn enum_struct_tests() {
    #[derive(serde::Serialize)]
    enum Struct {
        A { a: bool, b: bool },
    }

    assert_unsupported!(Struct, Struct::A { a: true, b: false });
}
