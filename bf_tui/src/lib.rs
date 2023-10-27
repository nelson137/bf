#![deny(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use std::io::Stdout;

use ratatui::{backend::CrosstermBackend, terminal};

pub mod async_interpreter;

pub mod events;

pub mod lines;

#[macro_use]
mod macros;

#[cfg(test)]
pub mod test_utils;

pub mod widgets;

type Backend = CrosstermBackend<Stdout>;
pub type Terminal = terminal::Terminal<Backend>;
pub type Frame<'a> = terminal::Frame<'a, Backend>;
