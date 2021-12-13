pub const VERSION: &str = concat!(
    env!("VERGEN_BUILD_DATE"),
    " (",
    env!("VERGEN_GIT_SHA_SHORT"),
    ")"
);
