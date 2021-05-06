// TODO: we could use a Cow<str> and save copying in the extremely
// common case of a bare atoms or string literals that don't contain
// any escape characters. But that would tie the lifetime of the Sexp
// to the lifetime of the input. Which might be fine? I will have to
// think about the tradeoffs there.

#[derive(Debug)]
pub enum Sexp {
    Atom(String),
    List(Vec<Sexp>),
}
