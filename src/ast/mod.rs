#[cfg(test)]
#[macro_use]
mod test;

mod component;
mod function;
mod import;
mod kind;
mod main;
mod record;
pub(crate) mod utils;
mod variable;

pub use component::{
    Argument, Component, ComponentDefinition, Event, Loop, Property, PropertySource,
};
pub use function::Function;
pub use import::Import;
pub use kind::{Condition, VariableKind, VariableModifier, VariableValue};
pub use main::AST;
pub use record::{Field, Record};
pub use variable::{VariableDefinition, VariableInvocation};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("P1Error: {}", _0)]
    P1Error(#[from] ftd::p11::Error),

    #[error("ASTParseError: {doc_id}:{line_number} -> {message}")]
    ParseError {
        message: String,
        doc_id: String,
        line_number: usize,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn parse_error<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd::ast::Result<T>
where
    S1: Into<String>,
{
    Err(Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}
