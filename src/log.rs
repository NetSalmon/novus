const ROOT_NAME: &str = "kernel";

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::print!("\x1b[32m[debug {}]\x1b[0m {}\n", module_path!(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::print!("\x1b[33m[info {}]\x1b[0m {}\n", module_path!(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::print!("\x1b[31m[error {}]\x1b[0m {}\n", module_path!(), format_args!($($arg)*));
    };
}
