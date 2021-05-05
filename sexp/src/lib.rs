mod parser;
mod sexp;
mod tokenizer;

pub use crate::sexp::Sexp;
use parser::{parse_many_tokens, ParseError};
use std::ops::Range;
use tokenizer::{tokenize, TokenizationError};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnclosedParen(usize),
    ExtraCloseParen(usize),
    UnclosedString(usize),
    UnknownEscape(Range<usize>),
}
impl Error {
    fn of_tokenization_error(e: TokenizationError) -> Error {
        use TokenizationError::*;
        match e {
            UnclosedString(i) => Error::UnclosedString(i),
        }
    }
    fn of_parse_error(e: ParseError) -> Error {
        use ParseError::*;
        match e {
            UnclosedParen(i) => Error::UnclosedParen(i),
            ExtraCloseParen(i) => Error::ExtraCloseParen(i),
            UnknownEscape(range) => Error::UnknownEscape(range),
        }
    }
}

pub fn parse_many(input: &str) -> Result<Vec<Sexp>, Error> {
    let tokens = tokenize(input).map_err(Error::of_tokenization_error)?;
    parse_many_tokens(input, tokens).map_err(Error::of_parse_error)
}
