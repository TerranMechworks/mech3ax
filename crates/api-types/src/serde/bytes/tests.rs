use ::serde::{Deserialize, Serialize};
use serde_test::{Configure as _, Token, assert_tokens};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Bytes {
    #[serde(with = "super")]
    pub bytes: Vec<u8>,
}

const B_RAW: &[u8] = b"hello, world!";
const B_B64: &str = "aGVsbG8sIHdvcmxkIQ==";

#[test]
fn bytes_json() {
    let expected = format!(r#"{{"bytes":"{}"}}"#, B_B64);
    let b: Bytes = serde_json::from_str(&expected).unwrap();
    assert_eq!(b.bytes, B_RAW);
    let actual = serde_json::to_string(&b).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn bytes_compact() {
    let b = Bytes {
        bytes: B_RAW.to_vec(),
    };
    assert_tokens(
        &b.compact(),
        &[
            Token::Struct {
                name: "Bytes",
                len: 1,
            },
            Token::Str("bytes"),
            Token::ByteBuf(B_RAW),
            Token::StructEnd,
        ],
    );
}

#[test]
fn bytes_readable() {
    let b = Bytes {
        bytes: B_RAW.to_vec(),
    };
    assert_tokens(
        &b.readable(),
        &[
            Token::Struct {
                name: "Bytes",
                len: 1,
            },
            Token::Str("bytes"),
            Token::String(B_B64),
            Token::StructEnd,
        ],
    );
}
