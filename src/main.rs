#![allow(unused)] //DISABLE SHITTY CODE WARNS


use std::{env::Vars, fmt::Debug};

use crate::test::TreeNode;
use daggy::{self, petgraph::{graph::{node_index, Node}, visit::{GraphRef, IntoNeighborsDirected, IntoNodeReferences}, Graph}, walker::{self, Chain, Inspect, TakeWhile}, Children, Dag, NodeIndex, Walker};


pub(crate) type BuildHasher = fxhash::FxBuildHasher;
pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;
pub(crate) type IndexSet<K> = indexmap::IndexSet<K, BuildHasher>;

mod test;

#[macro_export]
macro_rules! mstr { //simplify string::from function
    ( $($x:expr)? ) =>{
        $( String::from($x) )+
    };
}

struct Globals {
    boolean: bool,
    first_string: String,
    second_string: String,
}

// workaround to get as close as rust lets you get to globals
fn set_globals() -> Globals {
    Globals {
        boolean: true,
        first_string: mstr!("I am"),
        second_string: mstr!(" a Global!"),
    }
}

fn return_true() -> bool {
    true
}



fn print_some_glob_stuff(mut g: Globals){
    g.first_string.push_str(" definitely");
    g.boolean = false;
    println!("{} {}", g.first_string, g.second_string);
}


fn print_dag(mut g: Dag<u32, u32>){
    for n in g.raw_nodes() {
        //runs once for every node in the network
    }

    let w1 = g.children(node_index(0));
    fn do_once<F>(mut func: F)
    where F: FnMut()
    {
        func();
    };

    

    let mut expensive_closure = | mut num1: &Dag<u32, u32>, mut num2: u32| -> bool {
        let a = true;
        return a;
    };



    let w2 =
        TakeWhile::new(w1, expensive_closure);

    for (e, n) in w1.iter(&g){
        print!("{:?} {:?}\n", e, n);
    }
}


fn main() {
    // let g = set_globals();
    // let mut this_var_is = "useless";

    // let mut s = String::from("hello");
    // s.push_str(", world!");
    // let int = 5;
    // if int == 5 {
    //     println!("{}", s);
    // }
    // let testnode = test::build_node(mstr!("I am a node!"));
    // println!("{}", testnode.strn);

    // print_some_glob_stuff(g);
    
    ///TAG testing

    print!("-----\n");

    let mut graph: Dag<u32, u32, u32> = daggy::Dag::new();
    let idx = graph.add_node(1);
    graph.add_child(idx, 1, 1);
    graph.add_child(idx, 1, 1);
    print!("{:?}\n", idx);
    print_dag(graph);

    


}
