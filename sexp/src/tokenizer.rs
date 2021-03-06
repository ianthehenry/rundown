use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenizationError {
    UnclosedString(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    OpenParen(usize),
    CloseParen(usize),
    BareAtom(Range<usize>),
    StringLiteral(Range<usize>),
}

#[derive(Copy, Clone)]
enum TokenizationState {
    Boring,
    InString(usize, bool),
    InBareAtom(usize),
}

enum CharacterInterpretation {
    Whitespace,
    BeginString,
    OpenParen,
    CloseParen,
    AtomCharacter,
}

fn interpret(c: char) -> CharacterInterpretation {
    use CharacterInterpretation::*;
    match c {
        '"' => BeginString,
        '(' => OpenParen,
        ')' => CloseParen,
        c if c.is_whitespace() => Whitespace,
        _ => AtomCharacter,
    }
}

impl CharacterInterpretation {
    fn should_end_atom(&self) -> bool {
        !matches!(self, CharacterInterpretation::AtomCharacter)
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, TokenizationError> {
    use CharacterInterpretation::*;
    use TokenizationState::*;
    let mut state = Boring;
    let mut tokens: Vec<Token> = Vec::new();
    for (i, character) in input.char_indices() {
        match state {
            InString(start, true) => state = InString(start, false),
            InString(start, false) => match character {
                '\\' => state = InString(start, true),
                '"' => {
                    tokens.push(Token::StringLiteral(start + 1..i));
                    state = Boring;
                }
                _ => (),
            },
            Boring | InBareAtom(_) => {
                let interpretation = interpret(character);

                if interpretation.should_end_atom() {
                    if let InBareAtom(start) = state {
                        tokens.push(Token::BareAtom(start..i))
                    }
                }
                match interpretation {
                    BeginString => state = InString(i, false),
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
                            state = InBareAtom(i)
                        }
                    }
                }
            }
        }
    }
    match state {
        Boring => Ok(tokens),
        InString(start, _) => Err(TokenizationError::UnclosedString(start)),
        InBareAtom(start) => {
            tokens.push(Token::BareAtom(start..input.len()));
            Ok(tokens)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::tokenize;
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
            1..6,
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
            2..20,
        ),
    ],
)
"
        );
    }

    #[test]
    fn arbitrary_string_escapes_tokenize() {
        k9::snapshot!(
            tokenize(r#" "\a" "#),
            "
Ok(
    [
        StringLiteral(
            2..4,
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
            8..13,
        ),
        CloseParen(
            14,
        ),
        StringLiteral(
            16..25,
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
            5..11,
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
            4..5,
        ),
        BareAtom(
            8..9,
        ),
    ],
)
"
        );
    }
}
