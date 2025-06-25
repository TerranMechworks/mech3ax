use crate::flags;
use serde_test::{assert_tokens, Configure as _, Token};

flags! {
    struct TestFlags : u8 {
        const FOO_BAR = 1 << 0;
        const SPAM_EGGS = 1 << 1;
    }
}

#[test]
fn flags_json() {
    let expected_value = TestFlags::from_bits_truncate(u8::MAX);
    let json = serde_json::to_string(&expected_value).unwrap();
    assert_eq!(json, r#"{"foo_bar":true,"spam_eggs":true}"#);
    let actual_value: TestFlags = serde_json::from_str(&json).unwrap();
    assert_eq!(actual_value, expected_value);
}

#[test]
fn flags_readable() {
    let value = TestFlags::FOO_BAR;
    assert_tokens(
        &value.readable(),
        &[
            Token::Struct {
                name: "TestFlags",
                len: 2,
            },
            Token::Str("foo_bar"),
            Token::Bool(true),
            Token::Str("spam_eggs"),
            Token::Bool(false),
            Token::StructEnd,
        ],
    );
}

#[test]
fn flags_compact() {
    let value = TestFlags::from_bits_truncate(u8::MAX);
    assert_tokens(&value.compact(), &[Token::U8(3)]);
}
