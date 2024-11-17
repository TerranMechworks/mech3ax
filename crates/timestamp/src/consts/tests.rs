use super::{_month, _year, date};

#[test]
fn month_valid() {
    for month in 1..=12 {
        assert_eq!(_month(month) as u8, month, "month");
        let date = date(2024, month, 1);
        assert_eq!(date.month() as u8, month, "date");
    }
}

#[test]
#[should_panic(expected = "invalid month")]
fn month_invalid_zero() {
    _month(0);
}

#[test]
#[should_panic(expected = "invalid month")]
fn date_month_invalid_zero() {
    date(2024, 0, 1);
}

#[test]
#[should_panic(expected = "invalid month")]
fn month_invalid_thirteen() {
    _month(13);
}

#[test]
#[should_panic(expected = "invalid month")]
fn date_month_invalid_thirteen() {
    date(2024, 13, 1);
}

#[test]
fn year_valid() {
    for year in [0, 1601, 1970, 2000, 2024, 9999] {
        assert_eq!(_year(year), year, "year");
        let date = date(year, 1, 1);
        assert_eq!(date.year(), year, "date");
    }
}

#[test]
#[should_panic(expected = "invalid year")]
fn year_invalid_negative() {
    _year(-1);
}

#[test]
#[should_panic(expected = "invalid year")]
fn date_year_invalid_negative() {
    date(-1, 1, 1);
}

#[test]
#[should_panic(expected = "invalid year")]
fn year_invalid_positive() {
    _year(10000);
}

#[test]
#[should_panic(expected = "invalid year")]
fn date_year_invalid_positive() {
    date(10000, 1, 1);
}

#[test]
#[should_panic(expected = "invalid date")]
fn date_invalid() {
    date(2024, 2, 31);
}
