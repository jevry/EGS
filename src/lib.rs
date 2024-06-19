/*
 * lib.rs
 * -------------------------
 * Author  : Kieran van Gelder
 * Id      : 14033623
 * Date    : 2024
 * Version : 0.1
 * -------------------------
 * The library file that collects all the other files.
 * to use egs you can add "use egs;" to your file.
 * 
 * 
 */



//import/re-export the modules
mod id;
mod enode;
mod eclass;
mod unionfind;
pub mod util;
pub mod pattern;
pub mod egraph;
//import/re-export the functions and structs from the imported modules
use id::Id;
use enode::Enode;
use unionfind::UnionFind;
use eclass::EClass;
pub use symbolic_expressions::{Sexp, parser};
pub use pattern::read_ruleset;
pub use util::pretty_print;
pub use egraph::EGraph;









//run these tests on your local machine
//test functions
#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope
    use egraph::EGraph;
    ///example of unionfind.rs
    #[test]
    fn union_find() {
        let n = 10;
        let mut uf = UnionFind::default();
        print!("base set:                   {:?}\n", uf);
        for _ in 0..n {
            uf.new_set();
        }

        // test the initial condition of everyone in their own set
        print!("after adding some nodes:    {:?}\n", uf);

        // build up one set
        uf.union(itoid!(0), itoid!(1));
        uf.union(itoid!(0), itoid!(2));
        uf.union(itoid!(0), itoid!(3));
        print!("after making 1 set:         {:?}\n", uf);

        // build up another set
        uf.union(itoid!(6), itoid!(7));
        uf.union(itoid!(6), itoid!(8));
        uf.union(itoid!(6), itoid!(9));
        print!("after making 1 more set:    {:?}\n", uf);

        uf.union(itoid!(2), itoid!(9));
        print!("after union:                {:?}\n", uf);

        // this compresses all paths
        for i in 0..n {
            uf.find_mut(itoid!(i));
        }

        print!("after compression:          {:?}\n", uf);

        // indexes:           0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        //let expected = vec![0, 0, 0, 0, 4, 5, 0, 0, 0, 0];
    }

    use std::fs::read_to_string;
    use symbolic_expressions::parser::parse_str;
    use crate::pattern::Rule;
    #[test]
    fn construct_rule(){
        static PATH: &str = "src/rulesets/";
        static FILENAME: &str = "rulesetA.txt";
        let filepath = format!("{PATH}{FILENAME}");

        for line in read_to_string(filepath).unwrap().lines() {
            let parts = line.split("->");
            let collection = parts.collect::<Vec<&str>>();
            if collection.len() > 1{
                if let Ok(lhs) = parse_str(collection[0]){
                    if let Ok(rhs) = parse_str(collection[1]){
                        let r = Rule::new_rule(lhs, rhs).unwrap();
                        print!("{:?}  ->  ", r.lhs);
                        print!("{:?}\n", r.rhs);
                    }
                }

            }
        }
    }

    /*-------------------- EGRAPH TESTS --------------------*/

    use symbolic_expressions::parser;
    use crate::pattern::read_ruleset;
    use crate::pattern::new_pattern;

    static PATH: &str = "src/testsuite/";
    static FILENAME: &str = "ints/example.txt";

    #[test] ///run this test function to see graph construction
    fn egraph_construction() {
        let filepath = format!("{PATH}{FILENAME}");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        print!("empty graph: {:?}\n", g);
        let root = g.insert_sexpr(sexp);

        print!("\nnew graph: ");
        g.print();
        print!("root: {:?}\n", root);
    }



    ///run this test function to see adding a new term to a constructed graph
    ///note that doing this like this breaks the congruence invariant
    #[test]
    fn egraph_editing() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();

        let altsexp: Sexp = parser::parse_str("(<< a 1)").unwrap();

        print!("\nextra term: {:?}\n", altsexp);
        
        g.insert_sexpr(altsexp);

        print!("\nedited graph: ");
        g.print();
    }

    #[test] ///run this test function to see adding unioning 2 nodes
    fn egraph_union() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        let altsexp: Sexp = parser::parse_str("(<< a 1)").unwrap();
        g.insert_sexpr(altsexp);

        print!("\nedited graph: ");
        g.print();

        print!("\nunioning eclass 3 and 4...\n\n");
        g.union(itoid!(2), itoid!(4));
        g.print();
    }

    /*-------------------- REWRITE TESTS --------------------*/

    pub(crate) type BuildHasher = fxhash::FxBuildHasher;
    pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;
    #[test] ///to test ematching
    fn egraph_matching() {
        let filepath = format!("{PATH}ints/const_mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();

        let sexp1 = parse_str("(* P_a 2)").unwrap();
        let patt = new_pattern(sexp1.clone()).unwrap();

        let mut dict = IndexMap::<Enode, Id>::default();

        for (_, c) in g.classes.clone(){
            print!("\nres = {:?}\n", g.match_pattern(&c, &patt, &mut dict));
        }
        print!("var map = {:?}\n\n", dict);

    }

    #[test] ///to test rewriting a graph once
    fn egraph_rewrite() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();

        let sexp1 = parse_str("(* P_x 2)").unwrap();
        let sexp2 = parse_str("(<< P_x 1)").unwrap();
        let r = &Rule::new_rule(sexp1, sexp2).unwrap();
        g.rewrite_lhs_to_rhs(r);
        g.print();
    }

    /// to test rewriting a graph multiple times
    /// default settings will cause the egraph to saturate after the 4th rewrite
    /// after the 5th rewrite edits will return 0 and the number of eclasses and enodes stops changing
    #[test]
    pub fn egraph_mass_rewrite() {
        let filepath = format!("{PATH}ints/example.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();


        let ruleset = &read_ruleset(&format!("src/rulesets/rulesetA.txt"));
        let edits1 = g.rewrite_ruleset(ruleset);
        let edits2 = g.rewrite_ruleset(ruleset);
        let edits3 = g.rewrite_ruleset(ruleset);
        let edits4 = g.rewrite_ruleset(ruleset);
        let edits5 = g.rewrite_ruleset(ruleset);
        g.print();
        print!("1st pass edits: {}\n", edits1);
        print!("2nd pass edits: {}\n", edits2);
        print!("3rd pass edits made: {}\n", edits3);
        print!("4th pass edits made: {}\n", edits4);
        print!("5th pass edits made: {}\n", edits5);
        print!("uf_size = {}\n", g.uf_len());
        print!("enodes = {}\n", g.n_enodes());
        print!("eclasses = {}\n", g.n_eclasses());

    }

    /*-------------------- EXTRACT TESTS --------------------*/

    #[test] ///rewrite a term and extract some random terms from the graph
    pub fn extract_random_terms(){
        let filepath = format!("{PATH}ints/example.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        let root_id = g.insert_sexpr(sexp);
        let ruleset = &read_ruleset(&format!("src/rulesets/rulesetA.txt"));
        g.rewrite_ruleset(ruleset);
        g.print();
        
        if let Some(cls) = g.get_eclass_cpy(root_id){
            for n in cls.nodes{
                print!("res: {}\n", extract_random(&g, n));
            }
        }
    }

    use rand::{thread_rng, Rng};
    /// randomly choses what enodes to return.
    /// is prone to overflowing the stack for large graphs with cycles
    fn extract_random(e: &EGraph, n: Enode) -> String{
        let mut rng = thread_rng();
        let mut ans = String::new();
        if n.len() == 0 {return format!(" {}", n.head);}
        ans.push_str(&format!(" ( {} ", n.head));

        for id in n.args{
            let cid = e.find(id);
            if let Some(c) = e.get_eclass_cpy(cid){
                
                let ran = rng.gen_range(0..c.nodes.len());
                let n2 = c.nodes[ran].clone();
                ans.push_str(&extract_random(e, n2));
            }
        }
        ans.push_str(&format!(")"));
        return ans;
    }


    #[test] ///proof that commutivity works
    fn extract_commutivity(){
        let filepath = &format!("{PATH}ints/mult.txt");
        let rulepath = &format!("src/rulesets/commutivity_test.txt");
        let iter = 1;
        rewrite_extract(filepath, rulepath, iter, true);
    }

    #[test] ///proof that multiple rewrites work
    fn extract_chain(){
        //chain is deliberately built to require 4 iterations to find the
        //optimal rewrite
        let filepath = &format!("{PATH}chain.txt");
        let rulepath = &format!("src/rulesets/chain_ruleset.txt");
        let iter = 4;
        let res = rewrite_extract(filepath, rulepath, iter, false);
        assert!(parser::parse_str("e").unwrap() == res);
    }

    #[test] ///extracts the best found term from a simplification ruleset
    fn extract_zeros(){
        //(n x) -> x is the only used rule
        //for some reason it fails to finish the last simplify step
        //the issue is because of something in the rebuild function
        let filepath = &format!("{PATH}ints/add_zeros.txt");
        let rulepath = &format!("src/rulesets/recursive_rule.txt");
        let iter = 2;
        rewrite_extract(filepath, rulepath, iter, true);
    }

    #[test] ///extracts the best found term from peano 2+2
    fn extract_factorial(){
        //for some reason it finds 3+1 and fails to find 4
        //the issue is because of something in the rebuild function
        let filepath = &format!("{PATH}ints/factorial.txt");
        let rulepath = &format!("src/rulesets/factorial.txt");
        let iter = 2;
        rewrite_extract(filepath, rulepath, iter, true);
    }


    #[test] ///extracts the best found term from peano 2+2
    fn extract_peano(){
        //for some reason it finds 3+1 and fails to find 4
        //the issue is because of something in the rebuild function
        let filepath = &format!("{PATH}peano/sum.txt");
        let rulepath = &format!("src/rulesets/peano_ruleset.txt");
        let iter = 6;
        rewrite_extract(filepath, rulepath, iter, true);
    }

    #[test] ///extracts the best found term from example.txt
    fn extract_example(){
        let filepath = &format!("{PATH}ints/example.txt");
        let rulepath = &format!("src/rulesets/rulesetA.txt");
        let iter = 3;
        let res = rewrite_extract(filepath, rulepath, iter, false);
        assert!(parser::parse_str("a").unwrap() == res);
    }

    fn rewrite_extract(filepath: &str, rulepath: &str, rewrites: u32, show_g: bool)-> Sexp{
        let sexp: Sexp = parser::parse_file(filepath).unwrap();
        let mut g = EGraph::new();
        let root_id = g.insert_sexpr(sexp);
        let ruleset = &read_ruleset(rulepath);

        for _ in 0..rewrites{
            g.rewrite_ruleset(ruleset);
        }

        if show_g{
            g.print();
        }
        

        if let Some(str) =  g.extract_logical(root_id){
            if let Ok(res) = parser::parse_str(&str){
                pretty_print(&res, 10);
                return res
            }
        } else{
            print!("\nFailure to find extractable sexpr\n");
        }
        panic!("!!!extract or parsing of result failed!!!\n\n");
    }
}
