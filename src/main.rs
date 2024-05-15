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


use indexmap::{set::Union};

// #[derive(Eq, Hash, PartialEq, Default)]
// struct PatVar{
//     id: String
// }
// struct PatTerm<'a>{
//     head:String,
//     args:Vec< Union<'a, PatTerm<'a>, PatVar> >
// }

#[derive(Eq, Hash, PartialEq)]
enum Pattern {
    PatVar(String),
    PatTerm(String, Vec<Box<Pattern>>),
}

fn merge_consistent(dicts: Vec<IndexMap<Pattern, Term>>) -> Option<IndexMap<Pattern, Term>>{
    let mut newd: IndexMap<Pattern, Term> = IndexMap::<Pattern, Term>::default();

    for dict in dicts {
        for (k,v) in dict{
            if newd.contains_key(&k){
                if *newd.get(&k).unwrap() != v{
                    return None;
                }
            }else{
                newd.insert(k, v);
            }
        }
    }
    return Some(newd);
}

// fn match_pattern(t:Term,  p:Pattern) -> Option<IndexMap<Pattern, Term>> {
//     if let Pattern::PatVar(s) = p {
//         let mut m = IndexMap::<Pattern, Term>::default();
//         m.insert(Pattern::PatVar(s), t);
//         return m;
//     }
//     else if let Pattern::PatTerm(s, next) = p {
//         if t.head != s || t.args.len() != next.len(){
//             return None;
//         }
//         else {
//             let mut temp = Vec::<IndexMap<Pattern, Term>>::new();
//             for (t1,p1) in t.args.iter().zip(next){
//                 match_pattern(*t1,*p1);
//             }
    
//             // let v = [ match_pattern(t1,p1) for (t1,p1) in zip(t.args, p.args)];
    
//             return merge_consistent( temp );
//         }
//     }

//     return None;
// }



struct Rule {
    lhs: Pattern,
    rhs: Pattern
}
// impl Rule {
//     fn new(l: Term, r: Term) -> Rule{
//         let res = Rule {
//             lhs: l,
//             rhs: r
//         };
//         return res;
//     }
// }

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
            Point {x, y} =>println!("at {x} {y}")
        }
    }
}


