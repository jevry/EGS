#![allow(unused)] //DISABLE SHITTY CODE WARNS


use std::{env::Vars, fmt::Debug};


// use daggy::{self, petgraph::{graph::{node_index, Node},
//     visit::{GraphRef, IntoNeighborsDirected, IntoNodeIdentifiers, IntoNodeReferences}, Graph}, stable_dag::{self, StableDag}, walker::{self, Chain, Inspect, TakeWhile}, Children, NodeIndex, Walker
// };


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


//prints out the entire DAG starting from start_idx (0 if you want to print the full DAG)
// fn print_dag(g: StableDag<u32, u32>, start_idx: NodeIndex){
//     if (g.node_count()== 0) {return;}
//     print!("size = {:?}\n", g.node_count());
//     let mut simple_fn =
//                 |mut arg0: &StableDag<u32, u32>,
//                 mut arg1: &(daggy::EdgeIndex, NodeIndex)| -> bool {
//         let a = true;
//         print!("    child: {:?}\n", arg1.1);
//         return a;
//     };

//     for n_id in g.node_identifiers() {
//         print!("node {:?}\n", n_id);    
//         let w1 = g.children(n_id);
//         let w2 =
//             walker::TakeWhile::new(w1, simple_fn);
//         for (e, n) in w2.iter(&g){}
//         print!("\n");
//     }
// }



fn main() {

    // let mut graph: StableDag<u32, u32, u32>= stable_dag::StableDag::new();
    // let idx = graph.add_node(1);

    // graph.add_child(idx, 1, 1);
    // let (_, idx2) = graph.add_child(idx, 1, 1);
    // let (_, idx2) = graph.add_child(idx2, 1, 1);
    // graph.add_child(idx2, 1, 1);
    // graph.add_child(idx2, 1, 1);

    // print_dag(graph, idx);

    use util;
    let r = parser::parse_file("src/tes.txt").unwrap();

    pretty_print(&r, 10);

}
