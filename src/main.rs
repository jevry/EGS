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



fn main() {



    use util;
    let r = parser::parse_file("src/tes.txt").unwrap();

    pretty_print(&r, 10);

}
