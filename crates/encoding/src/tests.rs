use crate::windows1252_decode;
use std::borrow::Cow;

#[test]
fn ascii_is_borrowed() {
    let expected = "Hello, world!";
    let actual = windows1252_decode(expected.as_bytes());
    assert_eq!(expected, actual);
    assert!(matches!(actual, Cow::Borrowed(_)));
}

#[test]
fn cp1252_is_owned() {
    let expected = "Hellö, wörld!";
    let bytes = b"Hell\xf6, w\xf6rld!";
    let actual = windows1252_decode(bytes);
    assert_eq!(expected, actual);
    assert!(matches!(actual, Cow::Owned(_)));
}

#[test]
fn all_bytes() {
    let ascii = 0 as char..=127 as char;
    let cp1252 = "€\u{81}‚ƒ„…†‡ˆ‰Š‹Œ\u{8d}Ž\u{8f}\u{90}‘’“”•–—˜™š›œ\u{9d}žŸ\u{a0}¡¢£¤¥¦§¨©ª«¬\u{ad}®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþÿ".chars();
    let expected: String = ascii.chain(cp1252).collect();

    let bytes: Vec<_> = (u8::MIN..=u8::MAX).collect();
    let actual = windows1252_decode(&bytes);
    assert_eq!(expected, actual);
}
