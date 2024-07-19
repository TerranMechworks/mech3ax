mod bytes;
mod roundtrip_tests;
mod ser_tests;

pub(crate) use bytes::Bytes;

/// Over-simplified assert_matches implementation until it lands in Rust.
///
/// See https://github.com/rust-lang/rust/issues/82775.
#[macro_export]
macro_rules! assert_matches {
    ($left:expr, $right:pat $(,)?) => {
        match $left {
            $right => (),
            ref v => panic!(
                "assertion failed: `{:?}` does not match `{}`",
                v,
                stringify!($right)
            ),
        }
    };
}
