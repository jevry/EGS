#![allow(unused)] //DISABLE SHITTY CODE WARNS


use std::{env::Vars, fmt::Debug};



pub(crate) type BuildHasher = fxhash::FxBuildHasher;
pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;
pub(crate) type IndexSet<K> = indexmap::IndexSet<K, BuildHasher>;
mod util;
use util::pretty_print;
use symbolic_expressions::{Sexp, SexpError, parser};

#[macro_export]
macro_rules! mstr { //simplify string::from function
    ( $($x:expr)? ) =>{
        $( String::from($x) )+
    };
}

pub struct  Id{
    pub int: u32,
}

// pub struct EClass {
//     /// This eclass's id.
//     pub id: Id,
//     /// The equivalent enodes in this equivalence class.
//     pub nodes: Vec,
// }




fn main() {
    let path = "src/testsuite/";
    let filename = "ints/nested_add.txt";


    use util;

    let buf = format!("{path}{filename}");
    let r = parser::parse_file(&buf).unwrap();

    pretty_print(&r, 10);

}
