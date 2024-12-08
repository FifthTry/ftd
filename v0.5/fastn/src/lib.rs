#![deny(unused_crate_dependencies)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn;

// we are adding this so we can continue to use unused_crate_dependencies, as only
// main depends on tokio, and if we do not have the following unused_crate_dependencies
// complains
use tokio as _;

pub mod commands;
mod symbols;

pub enum Action {
    Read,
    Write,
}

pub enum OutputRequested {
    UI,
    Data,
}

pub fn full_filler(_i: Vec<String>) -> Vec<(String, Option<fastn_section::Document>)> {
    todo!()
}

pub async fn full_filler_async(_i: Vec<String>) -> Vec<(String, Option<fastn_section::Document>)> {
    todo!()
}
