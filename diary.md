# 2021-05-01

Well, I tried to use `pulldown-cmark-to-cmark`. But it doesn't... work. 

```
2 | use pulldown_cmark_to_cmark::fmt::cmark;
  |                              ^^^ private module
```

Copying the "stupicat" example.

`termbook` uses it, but references version `2.0.01`. Meanwhile the latest is `6.0.0`. So... yeah.

Gonna try something else.

Okay, I think I can do this with plain old `pulldown_cmark`.

---

Where is `map_fst`? I can't find it. And I don't know how to hoogle. I found a package called `itertools`, but I can't find that function in there.

---

What is the name of the `fst` function?

---

Hairy error message:

```
   Compiling rundown v0.1.0 (/Users/ian/src/rundown)
error[E0277]: the trait bound `Vec<(Event<'_>, std::ops::Range<usize>)>: From<Filter<Map<OffsetIter<'_>, [closure@src/main.rs:24:14: 28:10]>, [closure@src/main.rs:29:17: 32:10]>>` is not satisfied
  --> src/main.rs:33:10
   |
33 |         .into();
   |          ^^^^ the trait `From<Filter<Map<OffsetIter<'_>, [closure@src/main.rs:24:14: 28:10]>, [closure@src/main.rs:29:17: 32:10]>>` is not implemented for `Vec<(Event<'_>, std::ops::Range<usize>)>`
   |
   = help: the following implementations were found:
             <Vec<T> as From<&[T]>>
             <Vec<T> as From<&mut [T]>>
             <Vec<T> as From<BinaryHeap<T>>>
             <Vec<T> as From<Box<[T]>>>
           and 6 others
   = note: required because of the requirements on the impl of `Into<Vec<(Event<'_>, std::ops::Range<usize>)>>` for `Filter<Map<OffsetIter<'_>, [closure@src/m
```

I want to turn an `OffsetIter` into a `Vec`. Why doesn't this work?

Oh right. Because it's called `collect`. Not `into`. Whoops.

---

Okay. I can see how to iterate over parsed values with full reference to the underlying text. Which is all I really need.

But because there are so many overlapping ranges, I'll need to make a sort of "range merger". Since mostly I just care about "literal text string."

Yeah. Okay. I feel good about this.

# 2021-05-02

Some goofy things: I keep typing `->` instead of `=>`. I keep spacing the colon for type signatures.

---

Rust doesn't do type-directed disambiguation, so I have to write:

    let mut currentComponent: Component = Component::LiteralText(0..0);

Instead of:

    let mut currentComponent: Component = LiteralText(0..0);

Don't love that.

Similarly, having to qualify every single arm in a `match` expression. Ugh. That's horrible.

---

I hate so badly that match arms have to end in commas. That's horrible. That's almost as bad as the closure syntax. But `cargo fmt` showed me that I should just be using `{}` at all times. Which is tolerable.

---

I want to mutate something that is a part of an enum.

I don't know if this is allowed -- I want to mutate a range -- but it sort of feels like it should be. But I don't know how to borrow `currentRange` mutably.

    (Component::LiteralText(currentRange), Event::Start(Tag::CodeBlock(_))) => {

Ah. The problem was I needed to borrow the whole `currentComponent` mutably -- it wasn't that this "arm" was immutable, it was that the whole thing was immutable.

So here's a simple thing I'm trying to do:

```rust
let mut vec: Vec<Component> = Vec::new();
let mut currentComponent: Component = Component::LiteralText(0..0);

let moveOn = |newComponent| {
    vec.push(currentComponent);
    currentComponent = newComponent;
};

for (event, offset) in parser.into_offset_iter() {
    match (&mut currentComponent, event) {
        (Component::LiteralText(currentRange), Event::Start(Tag::CodeBlock(_))) => {
            currentRange.start = 1;
            moveOn(Component::Code(String::new(), String::new()));
            println!("transition to code phase");
        }
```

I want to make a helper to push the current element *and* set a new one. So I don't have to forget. But this seems... impossible?

Oh, ha. I've been using `camelCase` this whole time. I forgot Rust is snake case, until it warned me.

---

Why the hell do I have to clone `widest_range` in order to index with it? why does indexing need to take ownership of the range?

```rust
let widest_range = current_range.start..offset.start;
let new_end = match input[widest_range.clone()].rfind('\n') {
    None => offset.start,
    Some(index) => widest_range.start + index,
};
```

I guess the range winds up as a component of the slice? I don't get it. Why isn't `Range` copy?

---

Man; I keep typing `match ... with`. I'll get used to it...

---

I wanted my modules to be capitalized, like OCaml.

---

This is weird: `use` statements aren't inherited by child modules??

---

```
error[E0446]: private type `Component<'_>` in public interface
  --> src/main.rs:31:5
   |
16 | enum Component<'a> {
   | ------------------ `Component<'_>` declared as private
...
31 |     pub fn parse_all(input: &str) -> Box<dyn Iterator<Item = super::Component> + '_> {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ can't leak private type
```

Code looks like this:

```rust
#[derive(Debug)]
enum Component<'a> {
    LiteralText(&'a str),
    CodeBlock(CodeBlockFlavor<'a>, String),
}

mod parser {
    use core::ops::Range;
    use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
    use super::CodeBlockFlavor;

    enum Component<'a> {
        LiteralText(Range<usize>),
        CodeBlock(CodeBlockKind<'a>, Range<usize>, String),
    }

    pub fn parse_all(input: &str) -> Box<dyn Iterator<Item = super::Component> + '_> {
```

Which is to say, it's not the private type. It's the type in the parent.

Hmm, after renaming it... it doesn't seem like that's the issue? I'm very confused.

No, okay, I see. It's complaining that the actual outer `Component` type is private. Which it is. But it's perfectly visible to the module here -- the module itself is *not* public. All I'm trying to do is make a file-private function.

I thought Rust modules were like OCaml modules -- little namespaces. But they seem to be more than that. They seem to be things that exist and have meaning for the whole crate infrastructure. I don't want to make this function public, or the type public. I want "public" to mean "visible to things that can see me," but instead it seems to mean "visible to the entire world." I hate that.

I'm just trying to make a type that's private to a function. Is a module not the right way to do that?

`rustc explain` actually taught me the right way: `pub(crate)`. Weird!

---

Well this is just awful:

```
   Compiling rundown v0.1.0 (/Users/ian/src/rundown)
error[E0658]: use of unstable library feature 'exact_size_is_empty'
  --> src/main.rs:74:29
   |
74 |             if !final_range.is_empty() {
   |                             ^^^^^^^^
   |
   = note: see issue #35428 <https://github.com/rust-lang/rust/issues/35428> for more information

error: aborting due to previous error

For more information about this error, try `rustc --explain E0658`.
error: could not compile `rundown`

To learn more, run the command again with --verbose.
```

Googling eventually teaches me that Rust is Dumb, and for some reason is preferring some random trait instead of a method directly implemented.

I would assume the fix is:

    if !final_range.std::ops::Range::is_empty() {

But no, it seems I have to write:

    if !std::ops::Range::is_empty(final_range) {

Which, whatever, but why doesn't a method call allow a module disambiguation?

---

Okay. I now have something that can parse markdown coming in and print markdown going back out. And the markdown going out is identical to the markdown coming in, theoretically. And we also extract all code block contents.

It could basically be a de-tangler, at this point. A way to extract blocks of code from markdown.

# 2021-05-04

Tried using `ssexp` to parse info strings. It's *close*, but it's quite a bit more complicated than I need. Despite the complexity, I can't figure out any way to get it to support strings -- the `with_strings` "macro map" it has *sort of* supports strings, but there's no way to escape double quotes -- `"\""` -- within such strings. No way to suppress the delimiter, that I can find.

I just want something that gives me the OCaml view of a sexp. Lists and atoms. `lexpr` looks waaaay too complicated.

This is kind of a hilarious thing to rely on an external package for. It's such a trivial parser to write. Might be good exercise to do it in Rust.

I started doing that, a little bit, but it's late. I am kind of hilariously rusty at programming, regardless of the Rust slowing me down. This is a very trivial thing that I have definitely implemented before without trouble, but my brain is unable to wrap my head around it right now.

I got the second dose of the Pfizer COVID-19 vaccine about twelve hours ago. It *might* be hitting me.

# 2021-05-05

Man. I am having a lot of trouble getting used to colons in structs. I write `=` every time.

I tried to add `k9` as a dependency to write some snapshot tests. But I get this error:

```
= note: ld: library not found for -liconv
        clang-7: error: linker command failed with exit code 1 (use -v to see invocation)
```

Which I've definitely seen before, using `nix-shell -p`. I added `libiconv` to my list of `nix-shell -p` and that fixed it. I should probably write a `shell.nix` file...

---

Hmm. To parse, I tried to write something like this:

```rust
fn parse_list(tokens : &mut dyn Iterator<Item = Token>) -> Result<Vec<Sexp>, ParseError> {
    use Token::*;
    let mut sexps: Vec<Sexp> = Vec::new();
    for token in tokens {
        match token {
            CloseParen(_) => return Ok(sexps),
            OpenParen(start) => {
                match parse_list(tokens) {
                    Ok(list) => sexps.push(Sexp::List(list)),
                    Err(_) => return Err(ParseError::UnclosedParen(start))
                }
            }
            BareAtom(_range) =>
                sexps.push(Sexp::Atom("")),
            StringLiteral(_range) =>
                sexps.push(Sexp::Atom("")),
        }
    }
    Err(ParseError::UnclosedParen(0))
}
```

Basically I want the inner call to `parse_list` to advance the `tokens` iterator.

But I don't really think I can do that. Even though there's only one mutable borrow *at a time*.

I got it down to this simpler thing
:

```rust
fn parse_list<'a>(input: &'a str, mut tokens : &mut core::slice::Iter<Token>) -> Result<Vec<Sexp<'a>>, ParseError> {
    use Token::*;
    let mut sexps: Vec<Sexp> = Vec::new();
    for token in &mut tokens {
        match token {
            CloseParen(_) => return Ok(sexps),
            OpenParen(start) => {
                match parse_list(input, tokens) {
                    Ok(list) => sexps.push(Sexp::List(list)),
                    Err(_) => return Err(ParseError::UnclosedParen(*start))
                }
            }
            BareAtom(_range) =>
                sexps.push(Sexp::Atom("")),
            StringLiteral(_range) =>
                sexps.push(Sexp::Atom("")),
        }
    }
    Err(ParseError::UnclosedParen(0))
}
```

I don't understand why I need so many `mut`s there. The mutable borrow in the `for` is to get around an implicit call to `into_iter`. (I don't understand why I need that in the first place.)

Anyway, I think I'm going to abandon this approach and just explicitly return a "rest" iterator.

That worked just fine.
