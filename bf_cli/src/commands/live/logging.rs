use std::{env, fs::File, path::PathBuf};

use anyhow::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

const DEFAULT_DIRECTIVES: &str = if cfg!(debug_assertions) {
    "warn,bf=debug,"
} else {
    "warn,"
};

pub fn init_logging() -> Result<()> {
    // TODO: pick a better path
    let log_file_path = PathBuf::from("/tmp/bf.log");
    let log_file = File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_file_path)?;

    let directives = String::from(DEFAULT_DIRECTIVES)
        + env::var("RUST_LOG")
            .or_else(|_| env::var("BF_LOG"))
            .ok()
            .as_deref()
            .unwrap_or_default();
    let filter = EnvFilter::new(directives);

    let layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_filter(filter);

    tracing_subscriber::registry()
        .with(layer)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                ::tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: ::tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: ::tracing::Level::DEBUG, $ex)
    };
}
