#![allow(unused)] //DISABLE SHITTY CODE WARNS
//note: this code uses "return val;" instead of "val" to make it clearer what is happening


use std::{default, env::Vars, fmt::Debug, string};
use daggy::{petgraph::adj::List, Parents};
use hashbrown::HashMap;

mod id; use id::Id;
mod util;

use util::pretty_print;

//sexp stuff
use symbolic_expressions::{Sexp, SexpError, parser};


mod enode;
use enode::Enode;

mod unionfind;
use unionfind::UnionFind;

mod eclass;
use eclass::EClass;

mod egraph;
use egraph::EGraph;

mod pattern;


//we use indexmap instead of hashmap because indexmaps are more deterministic in their ordering
//but both should work
pub(crate) type BuildHasher = fxhash::FxBuildHasher;

pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;


use indexmap::{set::Union, map::Values};




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
    fn patt_matching_test2() {
        // let r = Rule {lhs: , rhs: };
    }


    #[test] //run this test function to see graph construction
    fn patt_matching_test() {
        struct Point {
            x: i32,
            y: i32,
        }
        let p = Point { x: 0, y: 7 };

        match p {
            Point { x, y } if x == y => println!("On the x axis at {x}"),
            Point { x: 0, y } => println!("On the y axis at {y}"),
            Point { x, y} =>println!("at {x} {y}")
        }
    }
}


