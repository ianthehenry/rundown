#[derive(Debug)]
pub enum Sexp {
    Atom(String),
    List(Vec<Sexp>),
}
