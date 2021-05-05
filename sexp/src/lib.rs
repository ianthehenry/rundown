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
    use k9;

    #[test]
    fn tokenize_atom() {
        k9::snapshot!(
            tokenize("hello"),
            "
Ok(
    [
        BareAtom(
            0..5,
        ),
    ],
)
"
        );
    }

    #[test]
    fn tokenize_string() {
        k9::snapshot!(
            tokenize(r#""hello""#),
            "
Ok(
    [
        StringLiteral(
            0..=6,
        ),
    ],
)
"
        );
    }

    #[test]
    fn tokenize_parens() {
        k9::snapshot!(
            tokenize(")()("),
            "
Ok(
    [
        CloseParen(
            0,
        ),
        OpenParen(
            1,
        ),
        CloseParen(
            2,
        ),
        OpenParen(
            3,
        ),
    ],
)
"
        );
    }

    #[test]
    fn string_escapes_work() {
        k9::snapshot!(
            tokenize(r#" "hello \"world\" \\" "#),
            "
Ok(
    [
        StringLiteral(
            1..=20,
        ),
    ],
)
"
        );
    }

    #[test]
    fn unterminated_strings() {
        k9::snapshot!(
            tokenize(r#" "hello "#),
            "
Err(
    UnclosedString(
        1,
    ),
)
"
        );
        k9::snapshot!(
            tokenize(r#" "hello\"#),
            "
Err(
    UnclosedString(
        1,
    ),
)
"
        );
    }

    #[test]
    fn tokenize_mix() {
        k9::snapshot!(
            tokenize(r#"(hello "world")"something" ) "#),
            "
Ok(
    [
        OpenParen(
            0,
        ),
        BareAtom(
            1..6,
        ),
        StringLiteral(
            7..=13,
        ),
        CloseParen(
            14,
        ),
        StringLiteral(
            15..=25,
        ),
        CloseParen(
            27,
        ),
    ],
)
"
        );
        k9::snapshot!(
            tokenize(r#" "hello\"#),
            "
Err(
    UnclosedString(
        1,
    ),
)
"
        );
    }

    #[test]
    fn quotes_can_snuggle_atoms() {
        k9::snapshot!(
            tokenize(r#"atom"string"atom"#),
            "
Ok(
    [
        BareAtom(
            0..4,
        ),
        StringLiteral(
            4..=11,
        ),
        BareAtom(
            12..16,
        ),
    ],
)
"
        );
    }

    #[test]
    fn normal_whitespace_separates_atoms() {
        k9::snapshot!(
            tokenize("a b\nc\td"),
            "
Ok(
    [
        BareAtom(
            0..1,
        ),
        BareAtom(
            2..3,
        ),
        BareAtom(
            4..5,
        ),
        BareAtom(
            6..7,
        ),
    ],
)
"
        );
    }

    #[test]
    fn unicode_whitespace_separates_atoms() {
        k9::snapshot!(
            // U+2009 is "THIN SPACE"
            // U+2029 is "PARAGRAPH SEPARATOR"
            tokenize("a\u{2009}b\u{2029}c"),
            "
Ok(
    [
        BareAtom(
            0..1,
        ),
        BareAtom(
            2..3,
        ),
        BareAtom(
            4..9,
        ),
    ],
)
"
        );
    }
}
