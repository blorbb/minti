#![feature(duration_constants, let_chains, lint_reasons, iter_intersperse)]
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
pub mod commands;
pub mod components;
pub mod contexts;
pub mod pages;
pub mod interpreter;
pub mod reactive;
pub mod time;
pub mod timer;

#[allow(unused_macros)]
macro_rules! wdbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        ::log::warn!("[{}:{}:{}]", ::std::file!(), ::std::line!(), ::std::column!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                ::log::warn!("[{}:{}:{}] {} = {:#?}",
                    ::std::file!(), ::std::line!(), ::std::column!(), ::std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::wdbg!($val)),+,)
    };
}
#[allow(unused_imports)]
pub(crate) use wdbg;