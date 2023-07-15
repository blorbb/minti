#![feature(duration_constants, let_chains, lint_reasons)]
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    // pedantic
    clippy::wildcard_imports,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::bool_to_int_with_if,
    clippy::similar_names,
    clippy::missing_panics_doc,
    // nursery
    clippy::option_if_let_else,
    clippy::future_not_send,
)]

pub mod app;
pub mod components;
pub mod pages;
pub mod utils;
