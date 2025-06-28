use crate::num;
use serde_test::{assert_tokens, Configure as _, Token};

num! {
    enum TestRepr : u8 {
        FooBar = 1,
        SpamEggs = 2,
    }
}

num! {
    enum TestPlain {
        FooBar = 3,
        SpamEggs = 4,
    }
}

#[test]
fn num_repr_json() {
    let expected_value = TestRepr::SpamEggs;
    let json = serde_json::to_string(&expected_value).unwrap();
    assert_eq!(json, "\"SpamEggs\"");
    let actual_value: TestRepr = serde_json::from_str(&json).unwrap();
    assert_eq!(actual_value, expected_value);
}

#[test]
fn num_repr_exchange() {
    let expected_value = TestRepr::SpamEggs;
    let exchange = mech3ax_exchange::to_vec(&expected_value).unwrap();
    // <EnumUnit><1i32>
    assert_eq!(exchange, &[80u8, 0, 0, 0, 1, 0, 0, 0] as &[u8]);
    let actual_value: TestRepr = mech3ax_exchange::from_slice(&exchange).unwrap();
    assert_eq!(actual_value, expected_value);
}

#[test]
fn num_repr_readable() {
    let value = TestRepr::FooBar;
    assert_tokens(
        &value.readable(),
        &[Token::UnitVariant {
            name: "TestRepr",
            variant: "FooBar",
        }],
    );
}

#[test]
fn num_repr_compact() {
    let value = TestRepr::FooBar;
    assert_tokens(
        &value.compact(),
        &[Token::UnitVariant {
            name: "TestRepr",
            variant: "FooBar",
        }],
    );
}

#[test]
fn num_plain_json() {
    let expected_value = TestPlain::SpamEggs;
    let json = serde_json::to_string(&expected_value).unwrap();
    assert_eq!(json, "\"SpamEggs\"");
    let actual_value: TestPlain = serde_json::from_str(&json).unwrap();
    assert_eq!(actual_value, expected_value);
}

#[test]
fn num_plain_exchange() {
    let expected_value = TestPlain::SpamEggs;
    let exchange = mech3ax_exchange::to_vec(&expected_value).unwrap();
    // <EnumUnit><1i32>
    assert_eq!(exchange, &[80u8, 0, 0, 0, 1, 0, 0, 0] as &[u8]);
    let actual_value: TestPlain = mech3ax_exchange::from_slice(&exchange).unwrap();
    assert_eq!(actual_value, expected_value);
}

#[test]
fn num_plain_readable() {
    let value = TestPlain::FooBar;
    assert_tokens(
        &value.readable(),
        &[Token::UnitVariant {
            name: "TestPlain",
            variant: "FooBar",
        }],
    );
}

#[test]
fn num_plain_compact() {
    let value = TestPlain::FooBar;
    assert_tokens(
        &value.compact(),
        &[Token::UnitVariant {
            name: "TestPlain",
            variant: "FooBar",
        }],
    );
}
