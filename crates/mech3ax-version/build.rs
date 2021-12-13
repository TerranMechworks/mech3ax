use vergen::{vergen, Config, ShaKind, TimeZone, TimestampKind};

fn main() {
    let mut config = Config::default();
    let build = config.build_mut();
    *build.enabled_mut() = true;
    *build.timestamp_mut() = true;
    *build.timezone_mut() = TimeZone::Utc;
    *build.kind_mut() = TimestampKind::DateOnly;
    *build.semver_mut() = false;
    let git = config.git_mut();
    *git.enabled_mut() = true;
    *git.branch_mut() = false;
    *git.commit_timestamp_mut() = false;
    *git.semver_mut() = false;
    *git.sha_mut() = true;
    *git.sha_kind_mut() = ShaKind::Short;
    vergen(config).unwrap();
}
