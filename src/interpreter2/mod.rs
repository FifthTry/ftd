#[cfg(test)]
#[macro_use]
mod test;

#[macro_export]
macro_rules! try_ok_state {
    ($e:expr) => {
        match $e {
            $crate::interpreter2::StateWithThing::State(s) => {
                return Ok($crate::interpreter2::StateWithThing::new_state(s))
            }
            $crate::interpreter2::StateWithThing::Thing(t) => t,
        }
    };
}

#[macro_export]
macro_rules! try_state {
    ($e:expr) => {
        match $e {
            $crate::interpreter2::StateWithThing::State(s) => {
                return $crate::interpreter2::StateWithThing::new_state(s)
            }
            $crate::interpreter2::StateWithThing::Thing(t) => t,
        }
    };
}

mod constants;
mod main;
mod main2;
mod tdoc;
mod things;
pub mod utils;

pub use constants::{
    FTD_ALIGN, FTD_ALIGN_BOTTOM_CENTER, FTD_ALIGN_BOTTOM_LEFT, FTD_ALIGN_BOTTOM_RIGHT,
    FTD_ALIGN_CENTER, FTD_ALIGN_LEFT, FTD_ALIGN_RIGHT, FTD_ALIGN_SELF, FTD_ALIGN_SELF_CENTER,
    FTD_ALIGN_SELF_END, FTD_ALIGN_SELF_START, FTD_ALIGN_TOP_CENTER, FTD_ALIGN_TOP_LEFT,
    FTD_ALIGN_TOP_RIGHT, FTD_BACKGROUND, FTD_BACKGROUND_SOLID, FTD_COLOR, FTD_COLOR_DARK,
    FTD_COLOR_LIGHT, FTD_IMAGE_SRC, FTD_IMAGE_SRC_DARK, FTD_IMAGE_SRC_LIGHT, FTD_LENGTH,
    FTD_LENGTH_PERCENT, FTD_LENGTH_PX, FTD_LENGTH_VALUE, FTD_OVERFLOW, FTD_OVERFLOW_SCROLL,
    FTD_OVERFLOW_VISIBLE, FTD_RESIZING, FTD_RESIZING_FILL_CONTAINER, FTD_RESIZING_FIXED,
    FTD_RESIZING_HUG_CONTENT, FTD_SPACING_MODE, FTD_SPACING_MODE_SPACE_AROUND,
    FTD_SPACING_MODE_SPACE_BETWEEN, FTD_SPACING_MODE_SPACE_EVENLY, FTD_TEXT_ALIGN,
    FTD_TEXT_ALIGN_CENTER, FTD_TEXT_ALIGN_END, FTD_TEXT_ALIGN_JUSTIFY, FTD_TEXT_ALIGN_START,
};
pub use main2::{interpret, Document, Interpreter, InterpreterState, StateWithThing};
pub use tdoc::TDoc;
pub use things::{
    component::{
        Argument, Component, ComponentDefinition, Event, EventName, Loop, Property, PropertySource,
    },
    default,
    expression::Expression,
    function::{Function, FunctionCall},
    kind::{Kind, KindData},
    or_type::{OrType, OrTypeVariant},
    record::{Field, Record},
    value::{PropertyValue, PropertyValueSource, Value},
    variable::{ConditionalValue, Variable},
    Thing,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("P1Error: {}", _0)]
    P1Error(#[from] ftd::p11::Error),

    #[error("OldP1Error: {}", _0)]
    OldP1Error(#[from] ftd::p1::Error),

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
    EvalexprError(#[from] ftd::evalexpr::EvalexprError),
}

pub type Result<T> = std::result::Result<T, Error>;
