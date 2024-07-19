use ::serde::{Deserialize, Serialize};
use serde_test::{assert_tokens, Configure as _, Token};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Time {
    #[serde(with = "super")]
    pub time: OffsetDateTime,
}

const T_RFC: &str = "1970-01-01T00:00:00Z";

#[test]
fn time_json() {
    let expected = format!(r#"{{"time":"{}"}}"#, T_RFC);
    let t: Time = serde_json::from_str(&expected).unwrap();
    assert_eq!(t.time, OffsetDateTime::UNIX_EPOCH);
    let actual = serde_json::to_string(&t).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn time_compact() {
    let t: Time = Time {
        time: OffsetDateTime::UNIX_EPOCH,
    };
    assert_tokens(
        &t.compact(),
        &[
            Token::Struct {
                name: "Time",
                len: 1,
            },
            Token::Str("time"),
            Token::BorrowedStr(T_RFC),
            Token::StructEnd,
        ],
    );
}

#[test]
fn time_readable() {
    let t: Time = Time {
        time: OffsetDateTime::UNIX_EPOCH,
    };
    assert_tokens(
        &t.readable(),
        &[
            Token::Struct {
                name: "Time",
                len: 1,
            },
            Token::Str("time"),
            Token::BorrowedStr(T_RFC),
            Token::StructEnd,
        ],
    );
}
