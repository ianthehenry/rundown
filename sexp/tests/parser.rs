use k9;
use sexp::parse_many;

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
fn string_escapes() {
    k9::snapshot!(
        parse_many(r#" "\n \"\u263A\" \t \\" "#),
        r#"
Ok(
    [
        Atom(
            "
 "☺" \t \",
        ),
    ],
)
"#
    );
}

#[test]
fn string_escapes_only_support_four_digit_unicode() {
    // TODO... but pretty minor
    k9::snapshot!(
        parse_many(r#" "\n \U1F642 \t" "#),
        "
Err(
    UnknownEscape(
        2..15,
    ),
)
"
    );
}

#[test]
fn bad_string_escape() {
    k9::snapshot!(
        parse_many(r#" "\a" "#),
        "
Err(
    UnknownEscape(
        2..4,
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
