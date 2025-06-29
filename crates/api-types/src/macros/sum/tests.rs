use crate::sum;
use serde_test::{assert_tokens, Configure as _, Token};

sum! {
    enum TestSum {
        Foo,
        Bar(u8),
    }
}

#[test]
fn sum_json_unit() {
    let expected_value = TestSum::Foo;
    let json = serde_json::to_string(&expected_value).unwrap();
    assert_eq!(json, "\"Foo\"");
    let actual_value: TestSum = serde_json::from_str(&json).unwrap();
    assert_eq!(actual_value, expected_value);
}

#[test]
fn sum_json_newtype() {
    let expected_value = TestSum::Bar(42);
    let json = serde_json::to_string(&expected_value).unwrap();
    assert_eq!(json, "{\"Bar\":42}");
    let actual_value: TestSum = serde_json::from_str(&json).unwrap();
    assert_eq!(actual_value, expected_value);
}

#[test]
fn sum_exchange_unit() {
    let expected_value = TestSum::Foo;
    let exchange = mech3ax_exchange::to_vec(&expected_value).unwrap();
    // <EnumUnit><0u32>
    assert_eq!(exchange, &[80u8, 0, 0, 0, 0, 0, 0, 0] as &[u8]);
    let actual_value: TestSum = mech3ax_exchange::from_slice(&exchange).unwrap();
    assert_eq!(actual_value, expected_value);
}

#[test]
fn sum_exchange_newtype() {
    let expected_value = TestSum::Bar(42);
    let exchange = mech3ax_exchange::to_vec(&expected_value).unwrap();
    // <EnumNewType><1u32><Struct><42u8>
    assert_eq!(
        exchange,
        &[81u8, 0, 0, 0, 1, 0, 0, 0, 10, 0, 0, 0, 42] as &[u8]
    );
    let actual_value: TestSum = mech3ax_exchange::from_slice(&exchange).unwrap();
    assert_eq!(actual_value, expected_value);
}

#[test]
fn sum_repr_unit_readable() {
    let value = TestSum::Foo;
    assert_tokens(
        &value.readable(),
        &[Token::UnitVariant {
            name: "TestSum",
            variant: "Foo",
        }],
    );
}

#[test]
fn sum_repr_unit_compact() {
    let value = TestSum::Foo;
    assert_tokens(
        &value.compact(),
        &[Token::UnitVariant {
            name: "TestSum",
            variant: "Foo",
        }],
    );
}

#[test]
fn sum_repr_newtype_readable() {
    let value = TestSum::Bar(42);
    assert_tokens(
        &value.readable(),
        &[
            Token::NewtypeVariant {
                name: "TestSum",
                variant: "Bar",
            },
            Token::U8(42),
        ],
    );
}

#[test]
fn sum_repr_newtype_compact() {
    let value = TestSum::Bar(42);
    assert_tokens(
        &value.compact(),
        &[
            Token::NewtypeVariant {
                name: "TestSum",
                variant: "Bar",
            },
            Token::U8(42),
        ],
    );
}
