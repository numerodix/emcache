// Declare sub modules
pub mod cmd;
pub mod driver;

// internal stuff
mod tests;  // needed to be part of the compilation unit in test mode
mod tests_bench;  // needed to be part of the compilation unit in test mode


// Export our public api
pub use self::driver::Driver;
