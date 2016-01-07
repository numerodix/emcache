pub const APP_NAME: &'static str = "emcache";
pub const APP_VERSION: &'static str = "0.1.0-dev";


pub fn get_version_string() -> String {
    format!("{} {}", APP_NAME, APP_VERSION)
}
