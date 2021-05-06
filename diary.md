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

Why the hell do I have to clone `widest_range` in order to index with it? Why does indexing need to take ownership of the range?

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

I started doing that, a little bit, but it's late. I am laughably rusty at programming, regardless of the Rust slowing me down. This is a very trivial thing that I have definitely implemented before without trouble, but my brain is unable to wrap my head around it right now.

I got the second dose of the Pfizer COVID-19 vaccine about twelve hours ago. It *might* be hitting me.

# 2021-05-05

Man. I am having a lot of trouble getting used to colons in structs. I write `=` every time.

---

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

I got it down to this simpler thing:

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

---

Now all I need to do is handle string escapes properly...

I found a crate from five years ago that *mostly* does what I want. It doesn't support unicode codepoints greater than `U+FFFF`, but whatever. It also gives no useful error message on a parse error. Easy enough to swap it out or write my own if I ever care.

---

This looks like a very nice way to parse command-line arguments:

    https://docs.rs/structopt/0.3.21/structopt

Once it comes time to make an actual executable.

But first: I should probably figure out how to make child processes.

---

Okay, doing "run this command; get this output" is pretty trivial.

Not trivial: repls (which I use a lot) and shared code "sessions" (something like org-babel).

I'm going to focus on repls first because it's more my immediate use case, and it *feels* more general even though, in a way, it's sort of a much simpler thing.

So I looked at `expect`, and I *think* it should work alright.

I made this little script:

```tcl
set timeout -1
log_user 0
spawn nix repl
match_max 100000

set prompt "nix-repl> "

expect $prompt
log_user 1
send_user $prompt
send -- "1 +\n2\n"
expect $prompt
send -- "pkgs = import <nixpkgs> {}\n"
expect $prompt
send -- "3 + 4\n"
expect $prompt
```

Running that gives me pretty much the output that I want. So I can "compile" something like this:

    ```
    nix-repl> 1 +
              2
    3

    nix-repl> pkgs = import <nixpkgs> {}

    nix-repl> 3 + 4
    7
    ```

Into the expect script above. `PS1="nix-repl> "`, `PS2="          "`. Just error if there's ever any ambiguity -- which you can easily check by making sure you can parse out the same set of "inputs" that you put into it.

But how do we pause if a repl session is split?

I really don't like the `expect` docs. And the web page is down. It seems like it's not really real software. But it's installed by default even on macOS, so.

I might have to learn some tcl. It's so *weird*. 

I don't know how to `expect` from within Rust. It doesn't seem that there are any bindings to `libexpect`. I can see that there is a crate called `reckon` which... implements a tiny subset that I don't think I can use. And doesn't seem to use a pty. Does expect use a pty? I don't know. I think it must, right? I will need to figure out a way to change its width.

I figured out a way to "pause" the script:

```tcl
set x 0
proc unpause [] {
  upvar x x
  send_user "trying to unpause"
  set x 1
}
trap unpause SIGUSR1

while {$x == 0} {
  sleep 1
}

send -- "3 + 4\n"
expect $prompt
expect eof
```

The sleep is gross, but in reality it would probably go very quickly. `rundown` isn't going to keep it paused.

Wait! Reading tcl docs taught me `vwait` -- which does exactly what I was trying to do, in a more real way. I think I can get by with this:

```tcl
global x
proc unpause [] {
  global x
  set x 0
}
trap unpause SIGUSR1

flush stdout
flush stderr
vwait x
```

I still wish I could just `raise SIGSTOP` and then sent `SIGCONT` -- although that might give me grief with buffering? In case of a nonblocking stdout? I have no idea how flushing works. So it might actually be better this way? In any case I can't find a `raise` equivalent in tcl, and I really don't want to shell out just to invoke `kill` on myself.

---

Huh. Should prompt detection be a regular expression? It seems like a reasonable option. `irb`, out of the box, has an incrementing count. `irb --simple-prompt` fixes it, but it might still be worth doing something about.

---

I also... I am being a little naïve here. It *seems* to work so nicely because I am printing to a terminal, and the terminal is handling things like `\r`. I might need something like https://crates.io/crates/vte to actually get decent output out of this.

# 2021-05-06

It occurs to me that the dumb "return the rest of the iterator" thing I was doing *can't* be necessary, and the problem must be the `for` loop. But since I'm using a `while let`/`.next()` now, there's absolutely no reason why I can't borrow mutably.

And it works!

But I have to declare tokens as mutable and as a mutable borrow:

```rust
mut tokens: &mut core::slice::Iter<'b, Token>,
```

And then when I use it:

```rust
OpenParen(start) => match parse_forms(input, &mut tokens)? {
```

Which is... weird? I want to just like clone the *mutable borrow*. I'm re-borrowing, and then something's gotta be getting implicitly `Deref`'d, right?

Ah! No. I can just do that. I don't know what I was thinking. Much simpler:

```rust
tokens: &mut core::slice::Iter<'b, Token>,
```

And then:

```rust
OpenParen(start) => match parse_forms(input, tokens)? {
```

Great. Great. That's what I want.

Now, is there a way to do the same thing from a `for` loop? Why does a `for` loop behave any differently than a `while let`/`.next()`?

Googling teaches me that it *seems* to be because `for` loops don't take iterators. They take `IntoIterator`s. And the API of `IntoIterator` requires that a `move` take place, even when the underlying type (here: `Iter`) already is an iterator.

I'm pretty happy with this explanation. And `while let`/`.next()` is really not that bad. So I'll just stick with that.

---

I ran `clippy` for the first time. It taught me the `matches!` macro, and pointed out that (following my iterator refactor above) I no longer need to explicitly name lifetimes.
