pub fn version() -> String {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let commit = option_env!("GIT_HASH").unwrap_or("unknown");
    let date = option_env!("BUILD_DATE").unwrap_or("unknown");

    format!("{name} {version} ({commit} {date})")
}
