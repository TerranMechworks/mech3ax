use time::{Date, Month, Time};

const fn _year(year: i32) -> i32 {
    match year {
        0..=9999 => year,
        _ => panic!("invalid year"),
    }
}

const fn _month(month: u8) -> Month {
    match month {
        1 => Month::January,
        2 => Month::February,
        3 => Month::March,
        4 => Month::April,
        5 => Month::May,
        6 => Month::June,
        7 => Month::July,
        8 => Month::August,
        9 => Month::September,
        10 => Month::October,
        11 => Month::November,
        12 => Month::December,
        0 | 13.. => panic!("invalid month"),
    }
}

/// Create a date.
///
/// # Panics
///
/// * If `year` is not between 0 and 9999 (inclusive).
/// * If `month` is not between 1 and 12 (inclusive).
/// * If the date is invalid.
pub(crate) const fn date(year: i32, month: u8, day: u8) -> Date {
    let year = _year(year);
    let month = _month(month);
    // poor unwrap, as `unwrap()` is not const
    match Date::from_calendar_date(year, month, day) {
        Ok(d) => d,
        Err(_e) => panic!("invalid date"),
    }
}

pub(crate) const fn time(hour: u8, minute: u8, second: u8, nanosecond: u32) -> Time {
    match Time::from_hms_nano(hour, minute, second, nanosecond) {
        Ok(t) => t,
        Err(_e) => panic!("invalid time"),
    }
}

#[cfg(test)]
mod tests;
