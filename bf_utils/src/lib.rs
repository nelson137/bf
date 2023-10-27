#![deny(clippy::cargo)]
#![warn(clippy::nursery)]
#![allow(clippy::multiple_crate_versions, clippy::option_if_let_else)]

#[macro_use]
mod macros;

pub mod hash;

pub mod sync;
