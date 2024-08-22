use time::OffsetDateTime;

fn main() {
    let ver = std::env::var("MECH3AX_VERSION").unwrap_or_else(|_e| "v0.0.0-test".to_string());
    let now = OffsetDateTime::now_utc();
    println!(
        "cargo::rustc-env=MECH3AX_BUILD_INFO={} ({:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z)",
        ver,
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    );
}
