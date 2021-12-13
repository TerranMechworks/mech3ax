use time::OffsetDateTime;

fn main() {
    let now = OffsetDateTime::now_utc();
    println!(
        "cargo:rustc-env=MECH3AX_BUILD_DATE={:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
}
