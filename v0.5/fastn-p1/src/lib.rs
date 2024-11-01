#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_p1;

#[cfg(test)]
mod debug;
mod error;
mod section;
mod unresolved;

pub use error::SingleError;
pub use section::{
    AliasableIdentifier, HeaderValue, Identifier, Kind, KindedName, ModuleName, PackageName,
    QualifiedIdentifier, Section, SectionInit, Span, Spanned, Visibility, SES,
};

#[derive(Default, Debug)]
pub struct Fuel {
    #[allow(dead_code)]
    remaining: std::rc::Rc<std::cell::RefCell<usize>>,
}

pub enum PResult<T> {
    NotFound,
    Found(T),
    Error(SingleError),
    Errors(Vec<SingleError>),
    FoundWithErrors {
        partial: T,
        errors: Vec<SingleError>,
    },
}

#[derive(Default)]
pub struct ParserEngine {
    pub doc_name: String,
    pub edits: Vec<Edit>,
}

pub struct Edit {
    pub from: usize,
    pub to: usize,
    pub text: Vec<char>,
}
