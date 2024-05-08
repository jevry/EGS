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

    //converts Sexp list to a Term vector
    pub fn sexpr2term(expr: Sexp) -> Vec<Term>{
        let mut buf: Vec<Term> = Vec::new();
        Term::rec_s2t(&mut buf, expr);
        return buf;
    }

    //resursive helper function
    fn rec_s2t(buf: &mut Vec<Term>, expr: Sexp) -> Id{
        let mut term: Term;
        if let Sexp::List(list) = expr { //operator
            let s = mstr!(list[0].to_string().trim_matches('"'));
            term = Term::new(s);
            let ida = Term::rec_s2t(buf, list[1].clone());
            let idb = Term::rec_s2t(buf, list[2].clone());

            term.args.push(ida);
            term.args.push(idb);
            term.args.sort();
        } else { //leaflet
            let s = mstr!(expr.to_string().trim_matches('"'));
            term= Term::new(s);

        }
        for (i, item) in buf.iter().enumerate(){
            if (*item == term) {
                return id!(i); //if we already have the term, return that term's id instead
            }
        }
        buf.push(term);
        return id!(buf.len()-1);
    }
    //Doesn't do anything yet
    pub fn term2sexp(term: Vec<Term>, buf: &mut Sexp){
        
    }
}