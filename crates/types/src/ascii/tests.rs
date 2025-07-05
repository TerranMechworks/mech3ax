use super::Ascii;
use crate::ConversionError;

macro_rules! ascii {
    ($v:literal) => {
        Ascii::new($v)
    };
}

macro_rules! ok {
    ($s:literal) => {
        Result::<String, ConversionError>::Ok($s.to_string())
    };
}

#[test]
fn ascii_debug_all_ascii() {
    let a = ascii!(b"hello!");
    let f = format!("{:?}", a);
    assert_eq!(f, "b\"hello!\"");
}

#[test]
fn ascii_debug_non_ascii() {
    let a = Ascii([0x00, 0xFF]);
    let f = format!("{:?}", a);
    assert_eq!(f, r#"b"\x00\xff""#);
}

#[test]
fn ascii_debug_always_has_no_newlines() {
    let a = ascii!(b"hello!");
    let f = format!("{:#?}", a);
    assert_eq!(f, "b\"hello!\"");
}

#[test]
fn ascii_split_combine() {
    let expected = ascii!(b"abcdefghijklmnopqrstuvwxyz_0123456789-ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let (l, r) = expected.split();
    let actual = Ascii::combine(&l, &r);
    assert_eq!(expected, actual);
}

#[test]
fn ascii_from_str_padded_valid() {
    let a = Ascii::<4>::from_str_padded("");
    assert_eq!(a, ascii!(b"\0\0\0\0"));
    let a = Ascii::<4>::from_str_padded("a");
    assert_eq!(a, ascii!(b"a\0\0\0"));
    let a = Ascii::<4>::from_str_padded("ab");
    assert_eq!(a, ascii!(b"ab\0\0"));
    let a = Ascii::<4>::from_str_padded("abc");
    assert_eq!(a, ascii!(b"abc\0"));
    let a = Ascii::<4>::from_str_padded("abcd");
    assert_eq!(a, ascii!(b"abc\0"));
    let a = Ascii::<4>::from_str_padded("abcde");
    assert_eq!(a, ascii!(b"abc\0"));
}

#[test]
#[should_panic(expected = "non-ASCII string")]
fn ascii_from_str_padded_invalid() {
    Ascii::<0>::from_str_padded("spam🎅eggs");
}

#[test]
fn ascii_from_str_padded_one() {
    let a = Ascii::<1>::from_str_padded("");
    assert_eq!(a, ascii!(b"\0"));
    let a = Ascii::<1>::from_str_padded("a");
    assert_eq!(a, ascii!(b"\0"));
    let a = Ascii::<1>::from_str_padded("ab");
    assert_eq!(a, ascii!(b"\0"));
}

#[test]
fn ascii_to_string_padded_vaild() {
    let s = ascii!(b"\0\0\0\0").to_str_padded();
    assert_eq!(s, ok!(""));
    let s = ascii!(b"a\0\0\0").to_str_padded();
    assert_eq!(s, ok!("a"));
    let s = ascii!(b"ab\0\0").to_str_padded();
    assert_eq!(s, ok!("ab"));
    let s = ascii!(b"abc\0").to_str_padded();
    assert_eq!(s, ok!("abc"));
}

#[test]
fn ascii_to_string_padded_invalid() {
    let s = ascii!(b"abcd").to_str_padded();
    assert_eq!(s, Err(ConversionError::Unterminated));
    let s = ascii!(b"ab\0\x01").to_str_padded();
    assert_eq!(s, Err(ConversionError::PaddingError("zeroes")));
    let s = ascii!(b"a\xBBc\0").to_str_padded();
    assert_eq!(s, Err(ConversionError::NonAscii(1)));
}

#[test]
fn ascii_from_str_node_name_valid() {
    let a = Ascii::<17>::from_str_node_name("");
    assert_eq!(a, ascii!(b"\0efault_node_name"));
    let a = Ascii::<17>::from_str_node_name("a");
    assert_eq!(a, ascii!(b"a\0fault_node_name"));
    let a = Ascii::<17>::from_str_node_name("abcd");
    assert_eq!(a, ascii!(b"abcd\0lt_node_name"));
    let a = Ascii::<17>::from_str_node_name("abcdefghijklmno");
    assert_eq!(a, ascii!(b"abcdefghijklmno\0e"));
    let a = Ascii::<17>::from_str_node_name("abcdefghijklmnop");
    assert_eq!(a, ascii!(b"abcdefghijklmnop\0"));
    let a = Ascii::<17>::from_str_node_name("abcdefghijklmnopq");
    assert_eq!(a, ascii!(b"abcdefghijklmnop\0"));
}

#[test]
#[should_panic(expected = "non-ASCII string")]
fn ascii_from_str_node_name_invalid() {
    Ascii::<0>::from_str_node_name("spam🎅eggs");
}

#[test]
fn ascii_node_name_valid() {
    let a = Ascii::<16>::node_name("");
    assert_eq!(a, ascii!(b"\0efault_node_nam"));
    let a = Ascii::<17>::node_name("");
    assert_eq!(a, ascii!(b"\0efault_node_name"));
    let a = Ascii::<18>::node_name("");
    assert_eq!(a, ascii!(b"\0efault_node_name\0"));

    let a = Ascii::<17>::node_name("a");
    assert_eq!(a, ascii!(b"a\0fault_node_name"));
    let a = Ascii::<17>::node_name("abcd");
    assert_eq!(a, ascii!(b"abcd\0lt_node_name"));
    let a = Ascii::<17>::node_name("abcdefghijklmno");
    assert_eq!(a, ascii!(b"abcdefghijklmno\0e"));
    let a = Ascii::<17>::node_name("abcdefghijklmnop");
    assert_eq!(a, ascii!(b"abcdefghijklmnop\0"));
    let a = Ascii::<17>::node_name("abcdefghijklmnopq");
    assert_eq!(a, ascii!(b"abcdefghijklmnop\0"));
}

#[test]
#[should_panic(expected = "non-ASCII string")]
fn ascii_node_name_invalid() {
    Ascii::<0>::node_name("spam🎅eggs");
}

#[test]
fn ascii_to_str_node_name_valid() {
    let s = ascii!(b"abc\0").to_str_node_name();
    assert_eq!(s, ok!("abc"));

    let s = ascii!(b"\0efault").to_str_node_name();
    assert_eq!(s, ok!(""));
    let s = ascii!(b"a\0fault").to_str_node_name();
    assert_eq!(s, ok!("a"));
    let s = ascii!(b"ab\0ault").to_str_node_name();
    assert_eq!(s, ok!("ab"));

    let s = ascii!(b"\0efault_node_name").to_str_node_name();
    assert_eq!(s, ok!(""));
    let s = ascii!(b"a\0fault_node_name").to_str_node_name();
    assert_eq!(s, ok!("a"));
    let s = ascii!(b"ab\0ault_node_name").to_str_node_name();
    assert_eq!(s, ok!("ab"));

    let s = ascii!(b"\0efault_node_name\0\0\0\0").to_str_node_name();
    assert_eq!(s, ok!(""));
    let s = ascii!(b"a\0fault_node_name\0\0\0\0").to_str_node_name();
    assert_eq!(s, ok!("a"));
    let s = ascii!(b"ab\0ault_node_name\0\0\0\0").to_str_node_name();
    assert_eq!(s, ok!("ab"));
}

#[test]
fn ascii_to_str_node_name_invalid() {
    let s = ascii!(b"abcd").to_str_node_name();
    assert_eq!(s, Err(ConversionError::Unterminated));
    let s = ascii!(b"ab\0\x01").to_str_node_name();
    assert_eq!(s, Err(ConversionError::PaddingError("node name")));
    let s = ascii!(b"ab\0\0").to_str_node_name();
    assert_eq!(s, Err(ConversionError::PaddingError("node name")));
    let s = ascii!(b"a\xBBc\0").to_str_node_name();
    assert_eq!(s, Err(ConversionError::NonAscii(1)));
}

#[test]
fn ascii_from_str_suffix_with_suffix() {
    let a = Ascii::<4>::from_str_suffix(".tif");
    assert_eq!(a, ascii!(b"\0tif"));
    let a = Ascii::<5>::from_str_suffix(".tif");
    assert_eq!(a, ascii!(b"\0tif\0"));
    let a = Ascii::<12>::from_str_suffix(".tif");
    assert_eq!(a, ascii!(b"\0tif\0\0\0\0\0\0\0\0"));

    let a = Ascii::<11>::from_str_suffix("foo bar.tif");
    assert_eq!(a, ascii!(b"foo bar\0tif"));
    let a = Ascii::<12>::from_str_suffix("foo bar.tif");
    assert_eq!(a, ascii!(b"foo bar\0tif\0"));
    let a = Ascii::<18>::from_str_suffix("foo bar.tif");
    assert_eq!(a, ascii!(b"foo bar\0tif\0\0\0\0\0\0\0"));

    let a = Ascii::<11>::from_str_suffix("foo.bar.tif");
    assert_eq!(a, ascii!(b"foo.bar\0tif"));
    let a = Ascii::<12>::from_str_suffix("foo.bar.tif");
    assert_eq!(a, ascii!(b"foo.bar\0tif\0"));
    let a = Ascii::<18>::from_str_suffix("foo.bar.tif");
    assert_eq!(a, ascii!(b"foo.bar\0tif\0\0\0\0\0\0\0"));
}

#[test]
fn ascii_from_str_suffix_no_suffix() {
    let a = Ascii::<12>::from_str_suffix("");
    assert_eq!(a, Ascii::zero());

    let a = Ascii::<3>::from_str_suffix("tif");
    assert_eq!(a, ascii!(b"ti\0"));
    let a = Ascii::<4>::from_str_suffix("tif");
    assert_eq!(a, ascii!(b"tif\0"));
    let a = Ascii::<12>::from_str_suffix("tif");
    assert_eq!(a, ascii!(b"tif\0\0\0\0\0\0\0\0\0"));

    let a = Ascii::<11>::from_str_suffix("foo bar_tif");
    assert_eq!(a, ascii!(b"foo bar_ti\0"));
    let a = Ascii::<12>::from_str_suffix("foo bar_tif");
    assert_eq!(a, ascii!(b"foo bar_tif\0"));
    let a = Ascii::<18>::from_str_suffix("foo bar_tif");
    assert_eq!(a, ascii!(b"foo bar_tif\0\0\0\0\0\0\0"));
}

#[test]
#[should_panic(expected = "non-ASCII string")]
fn ascii_from_str_suffix_invalid() {
    Ascii::<0>::from_str_suffix("spam🎅eggs");
}

macro_rules! str_suffix {
    ($a:literal, $s:literal) => {{
        let expected = ascii!($a);
        let s = expected.to_str_suffix().unwrap();
        assert_eq!(s, $s);
        let actual = Ascii::from_str_suffix(&s);
        assert_eq!(actual, expected);
    }};
}

#[test]
fn ascii_to_str_suffix_valid_suffix_and_padding() {
    str_suffix!(b"a\0d\0", "a.d");
    str_suffix!(b"abc\0def\0", "abc.def");
    str_suffix!(b"abc\0def\0\0\0\0", "abc.def");
}

#[test]
fn ascii_to_str_suffix_valid_suffix_only() {
    str_suffix!(b"a\0d", "a.d");
    str_suffix!(b"abc\0def", "abc.def");
}

#[test]
fn ascii_to_str_suffix_valid_padding_only() {
    str_suffix!(b"a\0\0\0", "a");
    str_suffix!(b"abc\0\0\0", "abc");
}

#[test]
fn ascii_to_str_suffix_valid_no_suffix_or_padding() {
    str_suffix!(b"a\0", "a");
    str_suffix!(b"abc\0", "abc");
}

#[test]
fn ascii_to_str_suffix_invalid() {
    let s = ascii!(b"").to_str_suffix();
    assert_eq!(s, Err(ConversionError::Unterminated));
    let s = ascii!(b"abcd").to_str_suffix();
    assert_eq!(s, Err(ConversionError::Unterminated));

    let s = ascii!(b"abc\0def\0\xBB").to_str_suffix();
    assert_eq!(s, Err(ConversionError::PaddingError("zeroes")));

    let s = ascii!(b"a\xBBc\0").to_str_suffix();
    assert_eq!(s, Err(ConversionError::NonAscii(1)));
    let s = ascii!(b"abc\0\xBB").to_str_suffix();
    assert_eq!(s, Err(ConversionError::NonAscii(4)));
}

#[test]
fn ascii_from_str_garbage_valid() {
    let a = Ascii::<12>::from_str_garbage("", &[]);
    assert_eq!(a, Ascii::zero());

    let a = Ascii::<12>::from_str_garbage("", b"ZYXWVUTSRQPONMLKJIHGFEDCBA");
    assert_eq!(a, ascii!(b"\0YXWVUTSRQPO"));

    let a = Ascii::<12>::from_str_garbage("", b"ZYXWVUTSRQPO");
    assert_eq!(a, ascii!(b"\0YXWVUTSRQPO"));

    let a = Ascii::<12>::from_str_garbage("", b"ZYXW");
    assert_eq!(a, ascii!(b"\0\0\0\0\0\0\0\0ZYXW"));

    let a = Ascii::<12>::from_str_garbage("abc", b"ZYXWVUTSRQPONMLKJIHGFEDCBA");
    assert_eq!(a, ascii!(b"abc\0VUTSRQPO"));

    let a = Ascii::<12>::from_str_garbage("abc", b"ZYXWVUTSRQPO");
    assert_eq!(a, ascii!(b"abc\0VUTSRQPO"));

    let a = Ascii::<12>::from_str_garbage("abc", b"ZYXW");
    assert_eq!(a, ascii!(b"abc\0\0\0\0\0ZYXW"));
}

#[test]
#[should_panic(expected = "non-ASCII string")]
fn ascii_from_str_garbage_invalid() {
    Ascii::<0>::from_str_garbage("spam🎅eggs", &[]);
}

#[test]
fn ascii_to_str_garbage_valid() {
    let (s, p) = ascii!(b"a\0").to_str_garbage().unwrap();
    assert_eq!(s, "a");
    assert_eq!(p, b"");

    let (s, p) = ascii!(b"a\0b").to_str_garbage().unwrap();
    assert_eq!(s, "a");
    assert_eq!(p, b"b");

    let (s, p) = ascii!(b"\0b").to_str_garbage().unwrap();
    assert_eq!(s, "");
    assert_eq!(p, b"b");
}

#[test]
fn ascii_to_str_garbage_invalid() {
    let s = ascii!(b"").to_str_garbage();
    assert_eq!(s, Err(ConversionError::Unterminated));
    let s = ascii!(b"abcd").to_str_garbage();
    assert_eq!(s, Err(ConversionError::Unterminated));

    let s = ascii!(b"\xAAbcd\0").to_str_garbage();
    assert_eq!(s, Err(ConversionError::NonAscii(0)));
}
