#![warn(rust_2018_idioms)]

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[derive(Debug)]
pub(super) enum BigParseErrorKind {
    LibError(String),
    Underflow,
}

pub(super) use BigParseErrorKind::{*};

#[derive(Debug)]
pub struct ParseBigError {
    pub(super) kind: BigParseErrorKind,
}

impl Error for ParseBigError {
    fn description(&self) -> &str {
        match &self.kind {
            LibError(_) => "library error",
            Underflow => "number can't be negative",
        }
    }
}

impl Display for ParseBigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            LibError(d) => write!(f, "{}: {}", self.description(), d),
            Underflow => write!(f, "{}", self.description()),
        }
    }
}

pub trait FromStrRadix where
    Self: std::marker::Sized,
{
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseBigError>;
}


