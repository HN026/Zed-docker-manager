/// Docker Manager for Zed -- a slash command extension for managing
/// Docker containers, images, and compose services from the Assistant panel.
///
/// The crate is split into pure-logic modules that are testable on any
/// target and a thin Zed glue layer compiled only for wasm32.

pub mod types;
pub mod docker;
pub mod commands;
pub mod completions;
pub mod testing;

#[cfg(target_arch = "wasm32")]
mod extension;
