#[cfg(feature="debug-server")]
pub const PIPE_PATH: &str = "PIPE";
#[cfg(feature="debug-server")]
pub static mut PIPE_FD: Option<i32> = None;

#[cfg(feature="debug-server")]
pub fn debug_print<S: AsRef<str>>(msg: S) {
    match unsafe{PIPE_FD} {
        None => panic!("Debug descriptor not open"),
        Some(fd) => {
            use nix::unistd::write;
            write(fd, msg.as_ref().as_bytes())
                .expect("Failed to write to debug descriptor");
        }
    }
}

#[cfg(feature="debug-server")]
#[macro_export]
macro_rules! dprint {
    ($($args:tt)*) => {
        crate::debug_server::debug_print(format!($($args)*));
    }
}

#[cfg(feature="debug-server")]
#[macro_export]
macro_rules! dprintln {
    ($($arg:tt)*) => {
        let d = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::new(0, 0));
        crate::dprint!("[{}.{:06}] ", d.as_secs(), d.subsec_micros());
        crate::dprint!($($arg)*);
        crate::dprint!("\n");
    }
}
