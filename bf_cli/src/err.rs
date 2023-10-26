#[macro_export]
macro_rules! err {
    ($($args:tt)*) => {
        Into::<anyhow::Error>::into(anyhow::anyhow!($($args)*))
    };
}

#[macro_export]
macro_rules! err_print {
    () => {
        $crate::err!("failed to print")
    };
}

#[macro_export]
macro_rules! err_file_open {
    ($path:expr) => {
        $crate::err!("failed to open file: {}", $path.display())
    };
}

#[macro_export]
macro_rules! err_file_read {
    ($path:expr) => {
        $crate::err!("failed to read file: {}", $path.display())
    };
}

#[macro_export]
macro_rules! err_file_write {
    ($path:expr) => {
        $crate::err!("failed to write to file: {}", $path.display())
    };
}
