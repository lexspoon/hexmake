/// Print an error message and exit out
macro_rules! error_exit {
    ($fmt:literal $($arg:tt)*) => {{
        println!($fmt $($arg)*);
        std::process::exit(1)
    }};
}

/// Reexport the macro for crate-wide use
pub(crate) use error_exit;
