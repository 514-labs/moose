//! Determine if the application is running in production mode.
pub fn isprod() -> bool {
    !cfg!(debug_assertions)
}
