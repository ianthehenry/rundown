use std::ops::Range;

#[derive(Debug)]
pub enum Sexp<'a> {
    Atom(&'a str),
    List(Vec<Sexp<'a>>),
}

#[derive(Debug, PartialEq, Eq)]
enum TokenizationError {
    UnclosedString(usize),
}

#[derive(Debug, PartialEq, Eq)]
enum ParseError {
    UnclosedParen(usize),
    ExtraCloseParen(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnclosedParen(usize),
    ExtraCloseParen(usize),
    UnclosedString(usize),
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
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    OpenParen(usize),
    CloseParen(usize),
    BareAtom(Range<usize>),
    StringLiteral(Range<usize>),
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
    for (i, character) in input.char_indices() {
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
                        tokens.push(Token::StringLiteral(start + 1..i));
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

// TODO
fn resolve_string_literal(string: &str) -> &str {
    return string;
}

enum StopReason {
    CloseParen(usize),
    EndOfInput,
}

fn parse_forms<'a, 'b>(
    input: &'a str,
    mut tokens: core::slice::Iter<'b, Token>,
) -> Result<(Vec<Sexp<'a>>, core::slice::Iter<'b, Token>, StopReason), ParseError> {
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
            BareAtom(range) => sexps.push(Sexp::Atom(&input[range.clone()])),
            StringLiteral(range) => {
                sexps.push(Sexp::Atom(resolve_string_literal(&input[range.clone()])))
            }
        }
    }
    Ok((sexps, tokens, StopReason::EndOfInput))
}

fn parse_many_tokens(input: &str, tokens: Vec<Token>) -> Result<Vec<Sexp>, ParseError> {
    match parse_forms(input, tokens[..].into_iter())? {
        (_, _, StopReason::CloseParen(i)) => Err(ParseError::ExtraCloseParen(i)),
        (sexps, _, StopReason::EndOfInput) => Ok(sexps),
    }
}

pub fn parse_many(input: &str) -> Result<Vec<Sexp>, Error> {
    let tokens = tokenize(input).map_err(Error::of_tokenization_error)?;
    parse_many_tokens(input, tokens).map_err(Error::of_parse_error)
}

#[cfg(test)]
mod tokenizer_tests {
    use super::{tokenize};
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

#[cfg(test)]
mod parser_tests {
    use super::{parse_many};
    use k9;

    #[test]
    fn parse_atoms() {
        k9::snapshot!(
            parse_many("hello there"),
            r#"
Ok(
    [
        Atom(
            "hello",
        ),
        Atom(
            "there",
        ),
    ],
)
"#
        );
    }

    #[test]
    fn parse_lists() {
        k9::snapshot!(
            parse_many("(hello) () ((there) (we (go)))"),
            r#"
Ok(
    [
        List(
            [
                Atom(
                    "hello",
                ),
            ],
        ),
        List(
            [],
        ),
        List(
            [
                List(
                    [
                        Atom(
                            "there",
                        ),
                    ],
                ),
                List(
                    [
                        Atom(
                            "we",
                        ),
                        List(
                            [
                                Atom(
                                    "go",
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        ),
    ],
)
"#
        );
    }

    #[test]
    fn extra_close_paren() {
        k9::snapshot!(
            parse_many("hello )"),
            "
Err(
    ExtraCloseParen(
        6,
    ),
)
"
        );
    }

    #[test]
    fn missing_close_paren() {
        k9::snapshot!(
            parse_many("(hello"),
            "
Err(
    UnclosedParen(
        0,
    ),
)
"
        );
    }

    #[test]
    fn unterminated_string() {
        k9::snapshot!(
            parse_many(r#"("hello"#),
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
    fn imbalanced_parens() {
        k9::snapshot!(
            parse_many("((((()))"),
            "
Err(
    UnclosedParen(
        1,
    ),
)
"
        );
    }

    #[test]
    fn unicode() {
        k9::snapshot!(
            parse_many(r#"🙂 (🙃) "🧵" "#),
            r#"
Ok(
    [
        Atom(
            "🙂",
        ),
        List(
            [
                Atom(
                    "🙃",
                ),
            ],
        ),
        Atom(
            "🧵",
        ),
    ],
)
"#
        );
    }
}
