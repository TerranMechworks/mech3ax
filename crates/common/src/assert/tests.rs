use super::assert_all_zero;

#[test]
fn is_equal_to() {
    let ident = 1;
    assert_that!("foo", ident == 1, 0).unwrap();
    let err = assert_that!("foo", ident == 2, 0).unwrap_err();
    assert_eq!(
        format!("{:#?}", err),
        "Expected `foo` == 2, but was 1 (at 0)"
    );
}

#[test]
fn is_not_equal_to() {
    let ident = 1;
    assert_that!("foo", ident != 2, 0).unwrap();
    let err = assert_that!("foo", ident != 1, 0).unwrap_err();
    assert_eq!(format!("{:#?}", err), "Expected `foo` != 1, but was (at 0)");
}

#[test]
fn is_less_than() {
    let ident = 1;
    assert_that!("foo", ident < 2, 0).unwrap();
    let err = assert_that!("foo", ident < 1, 0).unwrap_err();
    assert_eq!(
        format!("{:#?}", err),
        "Expected `foo` < 1, but was 1 (at 0)"
    );
}

#[test]
fn is_less_than_or_equal_to() {
    let ident = 1;
    assert_that!("foo", ident <= 2, 0).unwrap();
    assert_that!("foo", ident <= 1, 0).unwrap();
    let err = assert_that!("foo", ident <= 0, 0).unwrap_err();
    assert_eq!(
        format!("{:#?}", err),
        "Expected `foo` <= 0, but was 1 (at 0)"
    );
}

#[test]
fn is_greater_than() {
    let ident = 1;
    assert_that!("foo", ident > 0, 0).unwrap();
    let err = assert_that!("foo", ident > 2, 0).unwrap_err();
    assert_eq!(
        format!("{:#?}", err),
        "Expected `foo` > 2, but was 1 (at 0)"
    );
}

#[test]
fn is_greater_than_or_equal_to() {
    let ident = 2;
    assert_that!("foo", ident >= 2, 0).unwrap();
    assert_that!("foo", ident >= 1, 0).unwrap();
    let err = assert_that!("foo", ident >= 4, 0).unwrap_err();
    assert_eq!(
        format!("{:#?}", err),
        "Expected `foo` >= 4, but was 2 (at 0)"
    );
}

#[test]
fn is_between() {
    let ident = 1;
    assert_that!("foo", 0 <= ident <= 1, 0).unwrap();
    assert_that!("foo", 1 <= ident <= 2, 0).unwrap();
    let err = assert_that!("foo", 2 <= ident <= 3, 0).unwrap_err();
    assert_eq!(
        format!("{:#?}", err),
        "Expected 2 <= `foo` <= 3, but was 1 (at 0)"
    );
}

#[test]
fn all_zero_index() {
    let err = assert_all_zero("foo", 42, &[3]).unwrap_err();
    assert_eq!(
        format!("{:#?}", err),
        "Expected `foo` to be zero, but byte 0 was 03 (at 42)"
    );

    let err = assert_all_zero("foo", 42, &[0, 3]).unwrap_err();
    assert_eq!(
        format!("{:#?}", err),
        "Expected `foo` to be zero, but byte 1 was 03 (at 42)"
    );

    let err = assert_all_zero("foo", 42, &[0, 255, 0]).unwrap_err();
    assert_eq!(
        format!("{:#?}", err),
        "Expected `foo` to be zero, but byte 1 was FF (at 42)"
    );
}
