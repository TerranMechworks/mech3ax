use super::{Ascii, garbage, make_garbage};

macro_rules! assert_garbage {
    ($v:ident, Ok(($s:literal, $p:literal))) => {{
        let expected = Ok(($s.to_string(), Some($p.to_vec())));
        let actual = garbage(&$v);
        assert_eq!(actual, expected, "{:?}", $v);
        let (n, p) = actual.unwrap();
        let u = make_garbage(&n, &p);
        assert_eq!(u, $v);
    }};
    ($v:ident, Ok(($s:literal, None))) => {{
        let expected = Ok(($s.to_string(), None));
        let actual = garbage(&$v);
        assert_eq!(actual, expected, "{:?}", $v);
        let (n, p) = actual.unwrap();
        let u = make_garbage(&n, &p);
        assert_eq!(u, $v);
    }};
    ($v:ident, Err($e:expr)) => {{
        let expected = Err($e);
        let actual = garbage(&$v);
        assert_eq!(actual, expected, "{:?}", $v);
    }};
}

#[test]
fn ascii_garbage_valid() {
    // no padding
    let v = Ascii::new(b"a\0");
    assert_garbage!(v, Ok(("a", None)));

    // zero padding
    let v = Ascii::new(b"a\0\0");
    assert_garbage!(v, Ok(("a", None)));

    // garbage padding
    let v = Ascii::new(b"a\0b");
    assert_garbage!(v, Ok(("a", b"b")));

    // no name
    let v = Ascii::new(b"\0b");
    assert_garbage!(v, Ok(("", b"b")));
}

#[test]
fn ascii_garbage_invalid() {
    let v = Ascii::new(b"");
    assert_garbage!(v, Err("missing zero terminator".to_string()));

    let v = Ascii::new(b"abcd");
    assert_garbage!(v, Err("missing zero terminator".to_string()));

    let v = Ascii::new(b"\xAAbcd\0");
    assert_garbage!(v, Err("invalid character at +0".to_string()));
}

const P: Option<Vec<u8>> = None;

#[test]
fn make_garbage_valid_no_padding() {
    let v: Ascii<12> = make_garbage("", &P);
    assert_eq!(v, Ascii::zero());

    let v: Ascii<12> = make_garbage("abcdefghijklmnopqrstuvwxyz", &P);
    assert_eq!(v, Ascii::new(b"abcdefghijk\0"));

    let v: Ascii<12> = make_garbage("abcdefghijkl", &P);
    assert_eq!(v, Ascii::new(b"abcdefghijk\0"));

    let v: Ascii<12> = make_garbage("abcdefghijk", &P);
    assert_eq!(v, Ascii::new(b"abcdefghijk\0"));
}

macro_rules! pad {
    ($p:literal) => {
        Some($p as &[u8])
    };
}

#[test]
fn make_garbage_valid_padding() {
    let v: Ascii<12> = make_garbage("", &pad!(b""));
    assert_eq!(v, Ascii::zero());

    let v: Ascii<12> = make_garbage("", &pad!(b"ZYXWVUTSRQPONMLKJIHGFEDCBA"));
    assert_eq!(v, Ascii::new(b"\0YXWVUTSRQPO"));

    let v: Ascii<12> = make_garbage("", &pad!(b"ZYXWVUTSRQPO"));
    assert_eq!(v, Ascii::new(b"\0YXWVUTSRQPO"));

    let v: Ascii<12> = make_garbage("", &pad!(b"YXWVUTSRQPO"));
    assert_eq!(v, Ascii::new(b"\0YXWVUTSRQPO"));

    let v: Ascii<12> = make_garbage("", &pad!(b"ZYXW"));
    assert_eq!(v, Ascii::new(b"\0\0\0\0\0\0\0\0ZYXW"));

    let v: Ascii<12> = make_garbage("abc", &pad!(b"ZYXWVUTSRQPO"));
    assert_eq!(v, Ascii::new(b"abc\0VUTSRQPO"));

    let v: Ascii<12> = make_garbage("abc", &pad!(b"VUTSRQPO"));
    assert_eq!(v, Ascii::new(b"abc\0VUTSRQPO"));

    let v: Ascii<12> = make_garbage("abc", &pad!(b"ZYXW"));
    assert_eq!(v, Ascii::new(b"abc\0\0\0\0\0ZYXW"));
}

#[test]
#[should_panic(expected = "non-ASCII string")]
fn make_garbage_invalid_not_ascii() {
    let _: Ascii<0> = make_garbage("spam🎅eggs", &P);
}
