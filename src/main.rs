#![allow(unused)] //DISABLE SHITTY CODE WARNS
//note: this code uses "return val;" instead of "val" to make it clearer what is happening


use std::{default, env::Vars, fmt::Debug};
use daggy::{petgraph::adj::List, Parents};
use hashbrown::HashMap;

mod id; use id::Id;
mod util;
use util::pretty_print;

//sexp stuff
use symbolic_expressions::{Sexp, SexpError, parser};


mod terms;
use terms::Term;

mod unionfind;
use unionfind::UnionFind;

//TODO LIST:
//unionfind (y)
//eclass (y)
//termnode (y)
//termhash (y)


// #[macro_export]
// macro_rules! mstr { //simplify String::from function
//     ( $($x:expr)? ) =>{
//         $( String::from($x) )+
//     };
// }

//we use hasmap instead of indexmap because indexmaps are more deterministic
pub(crate) type BuildHasher = fxhash::FxBuildHasher;

pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;
pub(crate) type IndexSet<K> = indexmap::IndexSet<K, BuildHasher>;



//   `memo` to map `Term` to their equivalence class
//`classes` to map equivalence class `Id` to the `EClass`
#[derive(Debug)]
struct EGraph{
    unionfind:    UnionFind,
    memo:         IndexMap<Term, Id>,  //memory to store future term IDs
    classes:      IndexMap<Id,EClass>,
    dirty_unions: Vec<Id>,
}
impl EGraph{
    // Build an empty EGraph
    fn new() -> EGraph{
        let g = EGraph{
            unionfind:    UnionFind::default(),
            memo:         IndexMap::<Term, Id>::default(),
            classes:      IndexMap::<Id,EClass>::default(), // Use array?
            dirty_unions: Vec::<Id>::new()
        };
        return g;
    }

    fn print(&self){
        print!("{:?}\n", self.unionfind);
        print!("memo\n");
        for t in self.memo.clone(){
            print!("{:?} \n", t);
        }
        print!("\nclasses\n");
        for t in self.classes.clone(){
            print!("{:?} \n", t);
        }
        print!("dirty_unions {:?}\n", self.dirty_unions);
    }

    //returns the canonical id
    fn find_root(&mut self, id:Id)-> Id{
        return self.unionfind.find_mut(id);
    }

    // Checks if 2 given terms are in the same class.
    // Panics if either of the terms aren't present in the egraph.
    fn in_same_class(&mut self, t1:Term, t2:Term) -> bool{
        let (_, id1) = self.memo.get_key_value(&t1).unwrap();
        let (_, id2) = self.memo.get_key_value(&t2).unwrap();
        return self.unionfind.in_same_set(*id1, *id2);
    }

    // canonicalizes the args of a given term, then returns it
    fn canonicalize(&mut self, term: &mut Term){
        let mut new = Vec::<Id>::new();
        for i in term.args.clone(){
            new.push(self.unionfind.find_mut(i));
        }
        term.args = new;
    }

    //finds the corresponding Eclass Id of a given term
    //returns None if it cant find the Eclass
    fn find_eclass(&mut self, term: &mut Term) -> Option<Id>{
        self.canonicalize(term);
        let pair = self.memo.get_key_value(term);
        if pair.is_none(){
            return None;
        }
        else{
            let (_, id) = pair.unwrap();
            return Some(self.find_root(*id));
        }
    }



    // Push a potentially new eclass to the graph
    fn push_eclass(&mut self, term:&mut Term) -> Id{
        let id = self.find_eclass(term);
        if id.is_some(){ //term already in the graph
            return id.unwrap();
        }
        let id = self.unionfind.new_set();
        let eclass = EClass::new(term.clone());


        self.memo.insert(term.clone(), id);
        for child in term.args.clone(){ // set parent pointers
            self.classes[usize::from(child)].parents.push(id);
        }
        self.classes.insert(id, eclass);
        return id;
    }
}

#[derive(Clone, Debug)]
pub struct EClass {
    pub nodes: Vec<Term>, //Nodes part of this Eclass, Sexp = List([symbol, child1, childB])
    pub parents: Vec<Id>  //Parent Eclasses that point towards this Eclass
}
impl EClass {
    fn new(node: Term) -> EClass{
        let mut termvec = Vec::<Term>::new();
        termvec.push(node);
        let res = EClass {
            nodes: termvec,
            parents: Vec::<Id>::new()
        };
        return res;
    }
}

fn main() {
    // let path = "src/testsuite/";
    // let filename = "ints/nested_add.txt";

    // let buf = format!("{path}{filename}");
    // let r: Sexp = parser::parse_file(&buf).unwrap();

    // print!("{:?}\n", r);
    // pretty_print(&r, 10);

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
    fn egraph_construction() {
        let filepath = format!("{PATH}{FILENAME}");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let term = Term::sexpr2term(sexp);
        let mut g = EGraph::new();
        print!("empty graph: {:?}\n", g);
        for item in term{
            g.push_eclass(&mut item.clone());
        }
        print!("\nnew graph: ");
        g.print();
    }

}


