#![allow(unused)] //DISABLE SHITTY CODE WARNS
//note: this code uses "return val;" instead of "val" to make it clearer what is happening


use std::{default, env::Vars, fmt::Debug};
mod id; use id::Id;
mod util; use util::pretty_print;

//sexp stuff
use symbolic_expressions::{Sexp, SexpError, parser};

//we use hasmap instead of indexmap because indexmaps are more deterministic
use indexmap::{self, IndexMap};

mod unionfind;
use unionfind::UnionFind;

//TODO LIST:
//unionfind (y)
//eclass (n)
//termnode (y)
//termhash (n)


#[macro_export]
macro_rules! mstr { //simplify string::from function
    ( $($x:expr)? ) =>{
        $( String::from($x) )+
    };
}



pub struct EClass {
    pub id: Id, //own ID
    pub nodes: Vec<Sexp>, //Nodes part of this Eclass, Sexp = List([symbol, child1, childB])
    pub parents: Vec<Id>  //Parent Eclasses that point towards this Eclass
}

#[derive(Clone, Default)]
pub struct TermHash{
    pub h: IndexMap<Id, Term>
}
impl TermHash{
    fn term2hash(term: Term){

    }

    fn add(&self, term:Term){

    }
}

#[derive(Clone)]
pub struct Term {
    pub head: String,
    pub args: Vec<Id>
}
impl Term {
    fn sexp2term(expr: Sexp){

    }
    fn term2sexp(&self){
        
    }
}




fn main() {
    let path = "src/testsuite/";
    let filename = "ints/nested_add.txt";

    let buf = format!("{path}{filename}");
    let r: Sexp = parser::parse_file(&buf).unwrap();

    print!("{:?}\n", r);
    pretty_print(&r, 10);

}
