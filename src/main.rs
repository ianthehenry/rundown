use std::fs;
use std::io::Write as _;

#[derive(Debug)]
enum CodeBlockFlavor<'a> {
    Indented,
    Fenced {
        start_line: &'a str,
        end_line: Option<&'a str>,
        info_string: String,
    },
}

// indented code blocks do not include the leading
// indentation of the first line -- that's part of the
// previous node (i'm not sure about indented fences).
// not sure if it would be simpler to fix this up or not...
#[derive(Debug)]
enum Component<'a> {
    LiteralText(&'a str),
    CodeBlock(&'a str, CodeBlockFlavor<'a>, String),
}

mod parser {
    use super::CodeBlockFlavor;
    use super::Component as SuperComponent;
    use core::ops::Range;
    use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};

    enum Component<'a> {
        LiteralText(Range<usize>),
        CodeBlock(CodeBlockKind<'a>, Range<usize>, String),
    }

    pub(crate) fn to_components(input: &str) -> Box<dyn Iterator<Item = SuperComponent> + '_> {
        let parser = Parser::new_ext(input, Options::empty());

        // I don't like that I'm accumulating into a Vec here. I want
        // to write it as a folding_scan or something, but I don't
        // know how. Or just as a generator? I think the two-pass
        // approach is much cleaner but I don't know how to write it
        // without eagerly consuming the entire parser.
        let mut vec: Vec<Component> = Vec::new();
        let mut current_component: Component = Component::LiteralText(0..0);

        for (event, offset) in parser.into_offset_iter() {
            match (&mut current_component, event) {
                (Component::LiteralText(current_range), Event::Start(Tag::CodeBlock(kind))) => {
                    current_range.end = offset.start;
                    vec.push(current_component);
                    current_component = Component::CodeBlock(kind, offset, String::new());
                }
                (Component::LiteralText(current_range), _) => {
                    current_range.end = std::cmp::max(current_range.end, offset.end);
                }
                (Component::CodeBlock(_, _, _), Event::End(Tag::CodeBlock(_))) => {
                    vec.push(current_component);
                    current_component = Component::LiteralText(offset.end..offset.end);
                }
                (Component::CodeBlock(_, _, current_body), Event::Text(new_text)) => {
                    current_body.push_str(&new_text);
                }
                (Component::CodeBlock(_, _, _), event) => {
                    panic!("unexpected Event inside code block: {:?}", event)
                }
            }
        }

        // make sure we include all trailing whitespace
        match &mut current_component {
            Component::LiteralText(final_range) => final_range.end = input.len(),
            Component::CodeBlock(_, _, _) => panic!("no closing code end tag"),
        }
        vec.push(current_component);

        Box::new(vec.into_iter().map(move |component| match component {
            Component::LiteralText(range) => SuperComponent::LiteralText(&input[range]),
            Component::CodeBlock(kind, offset, body) => {
                let source = &input[offset.clone()];

                let flavor = match kind {
                    CodeBlockKind::Indented => CodeBlockFlavor::Indented,
                    CodeBlockKind::Fenced(info_string) => {
                        let end_of_first_line = source.find('\n').unwrap_or_else(|| {
                            panic!("fenced code block contains no newline! {}", source)
                        });
                        let start_line = &source[0..end_of_first_line];
                        let end_line = if offset.end == input.len() {
                            None
                        } else {
                            let start_of_last_line = source.rfind('\n').unwrap() + 1;
                            Some(&source[start_of_last_line..])
                        };
                        CodeBlockFlavor::Fenced {
                            start_line,
                            end_line,
                            info_string: info_string.into_string(),
                        }
                    }
                };
                SuperComponent::CodeBlock(source, flavor, body)
            }
        }))
    }
}

fn main() {
    let input = fs::read_to_string("input.md").expect("Something went wrong reading the file");
    let components = parser::to_components(&input);

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    for component in components {
        match component {
            Component::LiteralText(source) => {
                handle.write_all(source.as_bytes()).unwrap();
            }
            Component::CodeBlock(source, _, _) => {
                handle.write_all(source.as_bytes()).unwrap();
            }
        }
    }
}
