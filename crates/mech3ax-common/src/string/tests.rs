use super::*;

/// Over-simplified assert_matches implementation until it lands in Rust.
///
/// See https://github.com/rust-lang/rust/issues/82775.
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

#[test]
fn str_from_c_padded_with_zeros() {
    let buf = Ascii::new(b"spam eggs\0\0\0\0");
    let result = str_from_c_padded(&buf).unwrap();
    assert_eq!(result, "spam eggs");
}

#[test]
fn str_to_c_padded_with_zeros() {
    let mut buf: Ascii<13> = Ascii::zero();
    str_to_c_padded("spam eggs", &mut buf);
    assert_eq!(&buf.0, b"spam eggs\0\0\0\0");
}

#[test]
fn str_from_c_padded_at_length() {
    let buf = Ascii::new(b"spam eggs");
    let err = str_from_c_padded(&buf).unwrap_err();
    assert_matches!(err, ConversionError::Unterminated);
}

#[test]
fn str_to_c_padded_at_length() {
    let mut buf: Ascii<9> = Ascii::zero();
    str_to_c_padded("spam eggs", &mut buf);
    assert_eq!(&buf.0, b"spam egg\0");
}

#[test]
fn str_from_c_padded_with_non_zero() {
    let buf = Ascii::new(b"spam eggs\0ham");
    let err = str_from_c_padded(&buf).unwrap_err();
    assert_matches!(err, ConversionError::PaddingError(_));
}

#[test]
fn str_from_c_padded_non_ascii() {
    let buf = Ascii::new(b"spam\x80eggs\0");
    let err = str_from_c_padded(&buf).unwrap_err();
    assert_matches!(err, ConversionError::NonAscii(4));
}

#[test]
#[should_panic(expected = "Non-ASCII string")]
fn str_to_c_padded_non_ascii() {
    let mut buf: Ascii<9> = Ascii::zero();
    str_to_c_padded("spam🎅eggs", &mut buf);
}

#[test]
fn str_from_c_node_name_with_node_name() {
    let buf = Ascii::new(b"foo bar\0node_name\0\0\0");
    let result = str_from_c_node_name(&buf).unwrap();
    assert_eq!(result, "foo bar");
}

#[test]
fn str_to_c_node_name_with_node_name() {
    let mut buf: Ascii<20> = Ascii::zero();
    str_to_c_node_name("foo bar", &mut buf);
    assert_eq!(&buf.0, b"foo bar\0node_name\0\0\0");
}

#[test]
fn str_from_c_node_name_at_length() {
    let buf = Ascii::new(b"spam eggs");
    let err = str_from_c_node_name(&buf).unwrap_err();
    assert_matches!(err, ConversionError::Unterminated);
}

#[test]
fn str_to_c_node_name_at_length() {
    let mut buf: Ascii<9> = Ascii::zero();
    str_to_c_node_name("spam eggs", &mut buf);
    assert_eq!(&buf.0, b"spam egg\0");
}

#[test]
fn str_from_c_node_name_with_zeros() {
    let buf = Ascii::new(b"spam eggs\0\0\0\0");
    let err = str_from_c_node_name(&buf).unwrap_err();
    assert_matches!(err, ConversionError::PaddingError(_));
}

#[test]
fn str_from_c_node_name_non_ascii() {
    let buf = Ascii::new(b"foo\x80bar\0node_name\0\0\0");
    let err = str_from_c_node_name(&buf).unwrap_err();
    assert_matches!(err, ConversionError::NonAscii(3));
}

#[test]
#[should_panic(expected = "Non-ASCII string")]
fn str_to_c_node_name_non_ascii() {
    let mut buf: Ascii<9> = Ascii::zero();
    str_to_c_node_name("spam🎅eggs", &mut buf);
}

#[test]
fn str_from_c_suffix_with_suffix() {
    let buf = Ascii::new(b"foo bar\0tif\0");
    let result = str_from_c_suffix(&buf).unwrap();
    assert_eq!(result, "foo bar.tif");
}

#[test]
fn str_to_c_suffix_with_suffix() {
    let mut buf: Ascii<12> = Ascii::zero();
    str_to_c_suffix("foo bar.tif", &mut buf);
    assert_eq!(&buf.0, b"foo bar\0tif\0");
}

#[test]
fn str_from_c_suffix_no_suffix() {
    let buf = Ascii::new(b"foo bar\0\0");
    let result = str_from_c_suffix(&buf).unwrap();
    assert_eq!(result, "foo bar");
}

#[test]
fn str_to_c_suffix_no_suffix() {
    let mut buf: Ascii<9> = Ascii::zero();
    str_to_c_suffix("foo bar", &mut buf);
    assert_eq!(&buf.0, b"foo bar\0\0");
}

#[test]
fn str_from_c_suffix_completely_unterminated() {
    let buf = Ascii::new(b"foo bar");
    let err = str_from_c_suffix(&buf).unwrap_err();
    assert_matches!(err, ConversionError::Unterminated);
}

#[test]
fn str_from_c_suffix_with_suffix_unterminated() {
    let buf = Ascii::new(b"foo bar\0tif");
    let result = str_from_c_suffix(&buf).unwrap();
    assert_eq!(result, "foo bar.tif");
}

#[test]
fn str_from_c_suffix_with_suffix_with_non_zeros() {
    let buf = Ascii::new(b"foo bar\0tif\0ham\0");
    let err = str_from_c_suffix(&buf).unwrap_err();
    assert_matches!(err, ConversionError::PaddingError(_));
}

#[test]
fn str_from_c_suffix_non_ascii() {
    let buf = Ascii::new(b"foo\x80bar\0tif");
    let err = str_from_c_suffix(&buf).unwrap_err();
    assert_matches!(err, ConversionError::NonAscii(3));
}

#[test]
#[should_panic(expected = "Non-ASCII string")]
fn str_to_c_suffix_non_ascii() {
    let mut buf: Ascii<9> = Ascii::zero();
    str_to_c_suffix("spam🎅eggs", &mut buf);
}

#[test]
#[should_panic(expected = "padding overflows buffer")]
fn str_to_c_partition_too_much_padding() {
    let pad = vec![1, 2, 3, 4];
    let mut fill: Ascii<2> = Ascii::zero();
    str_to_c_partition("", &pad, &mut fill);
}

#[test]
fn str_to_c_partition_overlap() {
    let pad = vec![2, 3, 4];
    let mut fill: Ascii<4> = Ascii::zero();
    str_to_c_partition(".", &pad, &mut fill);
    assert_eq!(&fill.0, &[46, 0, 3, 4]);
}

#[test]
fn str_to_c_partition_gap() {
    let pad = vec![4];
    let mut fill: Ascii<4> = Ascii::zero();
    str_to_c_partition(".", &pad, &mut fill);
    assert_eq!(&fill.0, &[46, 0, 0, 4]);
}

#[test]
fn str_to_c_partition_fit() {
    let pad = vec![3, 4];
    let mut fill: Ascii<4> = Ascii::zero();
    str_to_c_partition(".", &pad, &mut fill);
    assert_eq!(&fill.0, &[46, 0, 3, 4]);
}

#[test]
fn str_from_c_partition_terminated_at_end() {
    let buf = Ascii::new(b"spam eggs\0");
    let (str, pad) = str_from_c_partition(&buf).unwrap();
    assert_eq!(str, "spam eggs");
    assert_eq!(pad.len(), 0);
}

#[test]
fn str_from_c_partition_terminated_at_start() {
    let buf = Ascii::new(b"\0spam eggs");
    let (str, pad) = str_from_c_partition(&buf).unwrap();
    assert_eq!(str, "");
    assert_eq!(pad, &buf.0[1..]);
}

#[test]
fn str_from_c_partition_terminated_at_mid() {
    let buf = Ascii::new(b"spam\0eggs");
    let (str, pad) = str_from_c_partition(&buf).unwrap();
    assert_eq!(str, "spam");
    assert_eq!(pad, b"eggs");
}

#[test]
fn str_from_c_partition_unterminated() {
    let buf = Ascii::new(b"spam eggs");
    let err = str_from_c_partition(&buf).unwrap_err();
    assert_matches!(err, ConversionError::Unterminated);
}

#[test]
fn str_from_c_partition_non_ascii() {
    let buf = Ascii::new(b"spam\x80eggs\0");
    let err = str_from_c_partition(&buf).unwrap_err();
    assert_matches!(err, ConversionError::NonAscii(4));
}

#[test]
#[should_panic(expected = "Non-ASCII string")]
fn str_to_c_partition_non_ascii() {
    let pad = vec![3, 4];
    let mut fill: Ascii<4> = Ascii::zero();
    str_to_c_partition("🎅", &pad, &mut fill);
}
