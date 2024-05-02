#![allow(unused)] //DISABLE SHITTY CODE WARNS
//note: this code uses "return val;" instead of "val" to make it clearer what is happening


use std::{default, env::Vars, fmt::Debug};
use daggy::petgraph::adj::List;
use hashbrown::HashMap;

mod id; use id::Id;
mod util; use util::pretty_print;

//sexp stuff
use symbolic_expressions::{Sexp, SexpError, parser};


mod terms;
use terms::Term;

mod unionfind;
use unionfind::UnionFind;

//TODO LIST:
//unionfind (y)
//eclass (n)
//termnode (y)
//termhash (n)


// #[macro_export]
// macro_rules! mstr { //simplify String::from function
//     ( $($x:expr)? ) =>{
//         $( String::from($x) )+
//     };
// }


#[derive(Clone, Default)]
pub struct TermHash{
    pub h: HashMap<Id, Term>
}
impl TermHash{
    fn term2hash(term: Vec<Term>) -> HashMap<Id, Term>{
        let r = HashMap::<Id, Term>::new();
        for (i, item) in term.iter().enumerate(){
            
        }
        return r;
    }

    fn add(&self, term:Term){

    }
}



//we use hasmap instead of indexmap because indexmaps are more deterministic
pub(crate) type BuildHasher = fxhash::FxBuildHasher;

pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;
pub(crate) type IndexSet<K> = indexmap::IndexSet<K, BuildHasher>;


//   `memo` to map `Term` to their equivalence class
//`classes` to map equivalence class `Id` to the `EClass`
struct EGraph{
    unionfind:    UnionFind,
    memo:         IndexSet<(Term,Id)>,  //we wanna use this IndexSet as a dict
    classes:      IndexSet<(Id,EClass)>, // Use array? //we wanna use this IndexSet as a dict
    dirty_unions: Vec<Id>,
}
impl EGraph{
    // Build an empty EGraph
    fn new() -> EGraph{
        let g = EGraph{
            unionfind:    UnionFind::default(),
            memo:         IndexSet::<(Term, Id)>::default(),
            classes:      IndexSet::<(Id,EClass)>::default(), // Use array?
            dirty_unions: Vec::<Id>::new()
        };
        return g;
    }
}

pub struct EClass {
    pub id: Id, //own ID
    pub nodes: Vec<Term>, //Nodes part of this Eclass, Sexp = List([symbol, child1, childB])
    pub parents: Vec<Id>  //Parent Eclasses that point towards this Eclass
}
impl EClass {
    fn new(idx: Id, node: Term) -> EClass{
        let mut t = Vec::<Term>::new();
        t.push(node);
        let res = EClass {
            id: idx,
            nodes: t, //Nodes part of this Eclass, Sexp = List([symbol, child1, childB])
            parents: Vec::<Id>::new()  //Parent Eclasses that point towards this Eclass
        };
        res
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

#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope
    static PATH: &str = "src/testsuite/";
    static FILENAME: &str = "ints/nested_add.txt";

    #[test] //run this test function to see term conversion
    fn term_conversion() {
        let buf = format!("{PATH}{FILENAME}");
        let sexp: Sexp = parser::parse_file(&buf).unwrap();
        pretty_print(&sexp, 10);

        let buf = Term::sexpr2term(sexp);
        print!("{:?}\n", buf);
    }



    #[test] //run this test function to see term conversion
    fn eclass_construction() {
        let buf = format!("{PATH}{FILENAME}");
        let sexp: Sexp = parser::parse_file(&buf).unwrap();
        // pretty_print(&sexp, 10);
        let buf = Term::sexpr2term(sexp);
        print!("{:?}\n", buf);
    }

}


