#[macro_export]
macro_rules! try_ok_state {
    ($e:expr) => {
        match $e {
            $crate::interpreter::StateWithThing::State(s) => {
                return Ok($crate::interpreter::StateWithThing::new_state(s))
            }
            $crate::interpreter::StateWithThing::Continue => {
                return Ok($crate::interpreter::StateWithThing::new_continue())
            }
            $crate::interpreter::StateWithThing::Thing(t) => t,
        }
    };
}

#[macro_export]
macro_rules! try_state {
    ($e:expr) => {
        match $e {
            $crate::interpreter::StateWithThing::State(s) => {
                return $crate::interpreter::StateWithThing::new_state(s)
            }
            $crate::interpreter::StateWithThing::Continue => {
                return $crate::interpreter::StateWithThing::new_continue()
            }
            $crate::interpreter::StateWithThing::Thing(t) => t,
        }
    };
}

#[cfg(test)]
#[macro_use]
mod test;
mod constants;
mod main;
pub mod prelude;
mod tdoc;
mod things;
pub mod utils;
pub use prelude::*;

pub use tdoc::{BagOrState, TDoc};
pub use things::expression;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("OtherError: {}", _0)]
    OtherError(String),

    #[error("P1Error: {}", _0)]
    P1Error(#[from] ftd::p1::Error),

    #[error("OldP1Error: {}", _0)]
    OldP1Error(#[from] ftd::ftd2021::p1::Error),

    #[error("ASTError: {}", _0)]
    ASTError(#[from] ftd::ast::Error),

    #[error("InvalidKind: {doc_id}:{line_number} -> {message}")]
    InvalidKind {
        doc_id: String,
        line_number: usize,
        message: String,
    },

    #[error("ValueNotFound: {doc_id}:{line_number} -> {message}")]
    ValueNotFound {
        doc_id: String,
        line_number: usize,
        message: String,
    },

    #[error("ParseIntError: {}", _0)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("ParseFloatError: {}", _0)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("ParseBoolError: {}", _0)]
    ParseBoolError(#[from] std::str::ParseBoolError),

    #[error("{doc_id}:{line_number} -> {message}")]
    ParseError {
        message: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("EvalexprError: {}", _0)]
    EvalexprError(#[from] fastn_grammar::evalexpr::EvalexprError),

    #[error("serde error: {source}")]
    Serde {
        #[from]
        source: serde_json::Error,
    },

    #[error("Invalid access: {message}, line_number: {line_number}")]
    InvalidAccessError { message: String, line_number: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
pub type ModuleThing = ftd::interpreter::things::ModuleThing;
