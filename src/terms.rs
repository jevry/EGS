use crate::id::Id;
use crate::util::{mstr, pretty_print};
use symbolic_expressions::{Sexp, SexpError, parser};

//the terms we use inside of the egraph, currently only supports Symbolic expressions
//though in theory it can support other data as well.
//commentary is written under the assumption of sexp

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