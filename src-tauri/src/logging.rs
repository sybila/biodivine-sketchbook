use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};

// Logging enabled by default
lazy_static! {
    static ref LOGGING_ENABLED: AtomicBool = AtomicBool::new(true);
}

/// Check if logging is enabled.
pub fn is_logging_enabled() -> bool {
    LOGGING_ENABLED.load(Ordering::SeqCst)
}

/// Disable logging.
pub fn disable_logging() {
    LOGGING_ENABLED.store(false, Ordering::SeqCst);
}

pub const DEBUG: u8 = 0;
pub const INFO: u8 = 8;
pub const WARNING: u8 = 16;
pub const ERROR: u8 = 32;

/// This is an initial attempt at a useful internal `log` macro. For now, it just logs a message
/// which is prefixed with its severity and location where the message was generated.
///
/// Later, we should hopefully do something like logging into a file,
/// filtering based on severity, and similar...
///
/// Also, this is conceptually similar to the `log` crate, which we ignore for now, but it could
/// be a good replacement for this module in the future (right now, the core benefit is that
/// this is easier to integrate and use for now, but it might be an issue once we need more
/// advanced features).
///
/// Also, the performance of this is not super great, but should be good enough once `DEBUG`
/// and `VERBOSE` output is disabled.
///
/// Also note that right now, IDEA does not seem to be able to recognize the file + line as
/// a clickable output element. So far, this feature seems to only work in Java and some other
/// pre-determined places. Hopefully, later they can actually provide click-able links in
/// such log output.
#[macro_export]
macro_rules! log {
    ($severity:tt, $($arg:tt)*) => {{
        if $crate::logging::is_logging_enabled() {
            print!("[level:{}][{}:{}] ", $severity, file!(), line!());
            println!($($arg)*);
        }
    }};
}

/// Version of `log` with `DEBUG` severity.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        use $crate::log;
        use $crate::logging::DEBUG;
        log!(DEBUG, $($arg)*);
    }};
}

/// Version of `log` with `VERBOSE` severity.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        use $crate::log;
        use $crate::logging::INFO;
        log!(INFO, $($arg)*);
    }};
}

/// Version of `log` with `WARNING` severity.
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        use $crate::log;
        use $crate::logging::WARNING;
        log!(WARNING, $($arg)*);
    }};
}

/// Version of `log` with `ERROR` severity.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use $crate::log;
        use $crate::logging::ERROR;
        log!(ERROR, $($arg)*);
    }};
}
