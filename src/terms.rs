use crate::id::{id, Id};
use crate::util::{mstr, pretty_print};


use symbolic_expressions::{Sexp, SexpError, parser};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Term {
    pub head: String,
    pub args: Vec<Id>
}
impl Term {
    pub fn new(string: String) -> Term {
        let test = Term  {
            head: string.clone(),
            args: Vec::new()
        };
        return test;
    }
}