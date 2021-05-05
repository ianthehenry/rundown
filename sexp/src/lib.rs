use std::ops::Range;
use std::ops::RangeInclusive;

#[derive(Debug)]
pub enum Sexp<'a> {
    Atom(&'a str),
    List(Vec<Sexp<'a>>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    UnclosedParen(usize),
    ExtraCloseParen(usize),
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    OpenParen(usize),
    CloseParen(usize),
    BareAtom(Range<usize>),
    StringLiteral(RangeInclusive<usize>),
}

#[derive(Debug, PartialEq, Eq)]
enum TokenizationError {
    UnclosedString(usize),
}

#[derive(Copy, Clone)]
enum TokenizationState {
    Boring,
    InString { start: usize, quotesy: bool },
    InBareAtom { start: usize },
}

enum CharacterInterpretation {
    Whitespace,
    BeginString,
    OpenParen,
    CloseParen,
    AtomCharacter,
}

fn interpret(character: char) -> CharacterInterpretation {
    use CharacterInterpretation::*;
    if character.is_whitespace() {
        Whitespace
    } else {
        match character {
            '"' => BeginString,
            '(' => OpenParen,
            ')' => CloseParen,
            _ => AtomCharacter,
        }
    }
}

impl CharacterInterpretation {
    fn should_end_atom(&self) -> bool {
        use CharacterInterpretation::*;
        match self {
            AtomCharacter => false,
            _ => true,
        }
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, TokenizationError> {
    use CharacterInterpretation::*;
    use TokenizationState::*;
    let mut state = Boring;
    let mut tokens: Vec<Token> = Vec::new();
    for (i, character) in input.chars().enumerate() {
        match state {
            InString {
                start,
                quotesy: true,
            } => {
                state = InString {
                    start,
                    quotesy: false,
                }
            }
            InString {
                start,
                quotesy: false,
            } => {
                match character {
                    '\\' => {
                        state = InString {
                            start,
                            quotesy: true,
                        }
                    }
                    '"' => {
                        // should we include the closing quote or not?
                        tokens.push(Token::StringLiteral(start..=i));
                        state = Boring;
                    }
                    _ => (),
                }
            }
            Boring | InBareAtom { .. } => {
                let interpretation = interpret(character);

                if interpretation.should_end_atom() {
                    if let InBareAtom { start } = state {
                        tokens.push(Token::BareAtom(start..i))
                    }
                }
                match interpretation {
                    BeginString => {
                        state = InString {
                            start: i,
                            quotesy: false,
                        }
                    }
                    OpenParen => {
                        tokens.push(Token::OpenParen(i));
                        state = Boring
                    }
                    CloseParen => {
                        tokens.push(Token::CloseParen(i));
                        state = Boring
                    }
                    Whitespace => state = Boring,
                    AtomCharacter => {
                        if let Boring = state {
                            state = InBareAtom { start: i }
                        }
                    }
                }
            }
        }
    }
    match state {
        Boring => Ok(tokens),
        InString { start, quotesy: _ } => Err(TokenizationError::UnclosedString(start)),
        InBareAtom { start } => {
            tokens.push(Token::BareAtom(start..input.len()));
            Ok(tokens)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token};

    #[test]
    fn single_atom() {
        assert_eq!(tokenize("hello"), Ok(vec![Token::BareAtom(0..5)]));
    }
}
