use crate::primitive_enum;

primitive_enum! {
    enum TestEnum: u8 {
        Foo = 1,
        Bar = 3,
    }
}

type Maybe = crate::Maybe<u8, TestEnum>;

#[test]
fn from_bits() {
    let r = TestEnum::from_bits(0);
    assert_eq!(r, None);

    let r = TestEnum::from_bits(1);
    assert_eq!(r, Some(TestEnum::Foo));

    let r = TestEnum::from_bits(2);
    assert_eq!(r, None);

    let r = TestEnum::from_bits(3);
    assert_eq!(r, Some(TestEnum::Bar));

    let r = TestEnum::from_bits(4);
    assert_eq!(r, None);
}

#[test]
fn maybe_display_normal() {
    let s = format!("{}", TestEnum::Foo.maybe());
    assert_eq!(s, "Foo (1)");
    let s = format!("{}", TestEnum::Bar.maybe());
    assert_eq!(s, "Bar (3)");
    let s = format!("{}", Maybe::new(0));
    assert_eq!(s, "<unknown> (0)");
}

#[test]
fn maybe_display_alt() {
    let s = format!("{:#}", TestEnum::Foo.maybe());
    assert_eq!(s, "Foo (1)");
    let s = format!("{:#}", TestEnum::Bar.maybe());
    assert_eq!(s, "Bar (3)");
    let s = format!("{:#}", Maybe::new(0));
    assert_eq!(s, "<unknown> (0)");
}

#[test]
fn maybe_debug_normal() {
    let s = format!("{:?}", TestEnum::Foo.maybe());
    assert_eq!(s, "Foo (1)");
    let s = format!("{:?}", TestEnum::Bar.maybe());
    assert_eq!(s, "Bar (3)");
    let s = format!("{:?}", Maybe::new(0));
    assert_eq!(s, "<unknown> (0)");
}

#[test]
fn maybe_debug_alt() {
    let s = format!("{:#?}", TestEnum::Foo.maybe());
    assert_eq!(s, "Foo (1)");
    let s = format!("{:#?}", TestEnum::Bar.maybe());
    assert_eq!(s, "Bar (3)");
    let s = format!("{:#?}", Maybe::new(0));
    assert_eq!(s, "<unknown> (0)");
}
