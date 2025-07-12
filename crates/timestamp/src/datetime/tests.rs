use super::DateTime;
use serde::{Deserialize, Serialize};
use serde_test::{Configure as _, Token, assert_tokens};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Time {
    time: DateTime,
}

impl Time {
    const UNIX_EPOCH: Self = Self {
        time: DateTime::UNIX_EPOCH,
    };
}

const T_RFC: &str = "1970-01-01T00:00:00.0Z";

#[test]
fn time_json() {
    let expected = format!(r#"{{"time":"{}"}}"#, T_RFC);
    let t: Time = serde_json::from_str(&expected).unwrap();
    assert_eq!(t, Time::UNIX_EPOCH);
    let actual = serde_json::to_string(&t).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn time_compact() {
    assert_tokens(
        &Time::UNIX_EPOCH.compact(),
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
    assert_tokens(
        &Time::UNIX_EPOCH.readable(),
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
