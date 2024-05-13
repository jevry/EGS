#![allow(unused)] //DISABLE SHITTY CODE WARNS
//note: this code uses "return val;" instead of "val" to make it clearer what is happening


use std::{default, env::Vars, fmt::Debug, string};
use daggy::{petgraph::adj::List, Parents};
use hashbrown::HashMap;

mod id; use id::Id;
mod util;
use indexmap::IndexSet;
use util::pretty_print;

//sexp stuff
use symbolic_expressions::{Sexp, SexpError, parser};


mod terms;
use terms::Term;

mod unionfind;
use unionfind::UnionFind;

mod eclass;
use eclass::EClass;

mod egraph;
use egraph::EGraph;


//we use hasmap instead of indexmap because indexmaps are more deterministic
pub(crate) type BuildHasher = fxhash::FxBuildHasher;

pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;


use indexmap::set::Union;
struct PatVar{
    id: String
}

struct PatTerm<'a>{
    head:String,
    args:Vec< Union<'a, PatTerm<'a>, PatVar> >
}

struct Pattern<'a>(Union<'a, PatTerm<'a>,PatVar>);







//TODO: empty this defunct code
fn main() {
    // let path = "src/testsuite/";
    // let filename = "ints/nested_add.txt";

    // let buf = format!("{path}{filename}");
    // let r: Sexp = parser::parse_file(&buf).unwrap();

    // print!("{:?}\n", r);
    // pretty_print(&r, 10);
}


//run these tests on your local machine
#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope
    static PATH: &str = "src/testsuite/";
    static FILENAME: &str = "ints/nested_add.txt";

    #[test] //run this test function to see graph construction
    fn egraph_construction() {
        let filepath = format!("{PATH}{FILENAME}");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        print!("empty graph: {:?}\n", g);
        g.term(sexp);

        print!("\nnew graph: ");
        g.print();
    }

    #[test] //run this test function to see adding a new term to a constructed graph
    fn egraph_editing() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.term(sexp);
        print!("\nnew egraph: ");
        g.print();

        let altsexp: Sexp = parser::parse_str("(<< a 1)").unwrap();

        print!("\nextra term: {:?}\n", altsexp);
        
        g.term(altsexp);

        print!("\nedited graph: ");
        g.print();
    }

    #[test] //run this test function to see adding a new term to a constructed graph
    fn egraph_union() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.term(sexp);
        let altsexp: Sexp = parser::parse_str("(<< a 1)").unwrap();
        g.term(altsexp);

        print!("\nedited graph: ");
        g.print();

        print!("\nunioning eclass 3 and 4...\n\n");
        g.union(itoid!(2), itoid!(4));
        g.print();
    }
}


