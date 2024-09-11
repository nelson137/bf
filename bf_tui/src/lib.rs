#![deny(clippy::cargo)]
#![warn(clippy::nursery)]
#![allow(clippy::multiple_crate_versions, clippy::option_if_let_else)]

use std::io::Stdout;

use ratatui::backend::CrosstermBackend;

pub mod async_interpreter;

pub mod events;

pub mod lines;

#[macro_use]
mod macros;

#[cfg(test)]
pub mod test_utils;

pub mod widgets;

type Backend = CrosstermBackend<Stdout>;
pub type Terminal = ratatui::Terminal<Backend>;
