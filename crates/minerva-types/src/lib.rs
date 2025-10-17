//! Shared domain types for the Minerva project.

pub mod board;
pub mod config;
pub mod events;
pub mod game;
pub mod telemetry;
pub mod time_control;
pub mod vision;

mod errors;

pub use errors::{MinervaError, Result};
