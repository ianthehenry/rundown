use crate::sexp::Sexp;
use crate::tokenizer::Token;
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    UnclosedParen(usize),
    ExtraCloseParen(usize),
    UnknownEscape(Range<usize>),
}

// TODO: I should write my own unescaper, with a proper error type
// that actually gives some information about the error that occurred.
fn resolve_string_literal(string: &str) -> Option<String> {
    unescape::unescape(string)
}

enum StopReason {
    CloseParen(usize),
    EndOfInput,
}

fn parse_forms<'a, 'b>(
    input: &'a str,
    mut tokens: core::slice::Iter<'b, Token>,
) -> Result<(Vec<Sexp>, core::slice::Iter<'b, Token>, StopReason), ParseError> {
    use Token::*;
    let mut sexps: Vec<Sexp> = Vec::new();

    while let Some(token) = tokens.next() {
        match token {
            CloseParen(index) => return Ok((sexps, tokens, StopReason::CloseParen(*index))),
            OpenParen(start) => match parse_forms(input, tokens)? {
                (_, _, StopReason::EndOfInput) => return Err(ParseError::UnclosedParen(*start)),
                (forms, rest, StopReason::CloseParen(_)) => {
                    sexps.push(Sexp::List(forms));
                    tokens = rest;
                }
            },
            BareAtom(range) => {
                let atom = input[range.clone()].to_string();
                sexps.push(Sexp::Atom(atom))
            }
            StringLiteral(range) => match resolve_string_literal(&input[range.clone()]) {
                None => return Err(ParseError::UnknownEscape(range.clone())),
                Some(string) => sexps.push(Sexp::Atom(string)),
            },
        }
    }
    Ok((sexps, tokens, StopReason::EndOfInput))
}

pub fn parse_many_tokens(input: &str, tokens: Vec<Token>) -> Result<Vec<Sexp>, ParseError> {
    match parse_forms(input, tokens[..].into_iter())? {
        (_, _, StopReason::CloseParen(i)) => Err(ParseError::ExtraCloseParen(i)),
        (sexps, _, StopReason::EndOfInput) => Ok(sexps),
    }
}
