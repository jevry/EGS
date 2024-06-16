#![allow(unused)] //DISABLE SHITTY CODE WARNS
//note: this code uses "return val;" instead of "val" to make it clearer what is happening


//This is the "hub" file that doesn't do anything on its own, it just collects all the other files.
//all the tests are stored here


mod id; use id::Id;
mod util;
use symbolic_expressions::{Sexp, parser};
use pattern::read_ruleset;

mod enode;
use enode::Enode;

mod unionfind;
use unionfind::UnionFind;

mod eclass;
use eclass::EClass;

mod egraph;
use egraph::EGraph;


mod pattern;



//TODO: empty this defunct code
fn main() {
    egraph_mass_rewrite();
}


pub fn egraph_mass_rewrite() {
    static PATH: &str = "src/testsuite/";
    let filepath = format!("{PATH}ints/example.txt");
    let sexp: Sexp = parser::parse_file(&filepath).unwrap();
    let mut g = EGraph::new();
    let root_id = g.insert_sexpr(sexp);
    let ruleset = &read_ruleset(format!("src/rulesets/rulesetA.txt"));
    g.rewrite_ruleset(ruleset);
    g.rebuild();
    g.rewrite_ruleset(ruleset);
    g.rebuild();
    g.rewrite_ruleset(ruleset);
    g.rebuild();
    g.rewrite_ruleset(ruleset);
    g.rebuild();
    g.print();
    

    let str =  g.extract_shortest(root_id).unwrap();
    if let Ok(res) = parser::parse_str(&str){
        util::pretty_print(&res, 5);
    }
}

//run these tests on your local machine
//test functions
#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope

    //example of unionfind.rs
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
    use util::pretty_print;
    use crate::pattern::Rule;
    #[test] //run this test function to see graph construction
    fn construct_rule(){
        static PATH: &str = "src/rulesets/";
        static FILENAME: &str = "patternB.txt";
        let filepath = format!("{PATH}{FILENAME}");

        for line in read_to_string(filepath).unwrap().lines() {
            let parts = line.split("->");
            let collection = parts.collect::<Vec<&str>>();
            if collection.len() > 1{
                if let Ok(lhs) = parse_str(collection[0]){
                    if let Ok(rhs) = parse_str(collection[1]){
                        let r = Rule::new_rule(lhs, rhs).unwrap();
                        print!("{:?}\n", r.lhs);
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

    #[test] //run this test function to see graph construction
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



    //run this test function to see adding a new term to a constructed graph
    //note that doing this like this breaks the congruence invariant
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

    #[test] //run this test function to see adding unioning 2 nodes
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

    pub(crate) type BuildHasher = fxhash::FxBuildHasher;
    pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;
    #[test] //to test ematching
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

    #[test] //to test rewriting a graph once
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

    #[test] //to test rewriting a graph multiple times
    pub fn egraph_mass_rewrite() {
        let filepath = format!("{PATH}ints/example.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();


        let ruleset = &read_ruleset(format!("src/rulesets/rulesetA.txt"));
        let edits = g.rewrite_ruleset(ruleset);

        print!("first pass edits: {}\n", edits);
        let edits = g.rewrite_ruleset(ruleset);

        print!("second pass edits: {}\n", edits);
        let edits = g.rewrite_ruleset(ruleset);
        print!("third pass edits made: {}\n", edits);
        g.rebuild();
        g.print();
    }



    #[test] //rewrite a term a few times and extract some random terms from the graph
    pub fn extract_random_terms(){
        let filepath = format!("{PATH}ints/example.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        let root_id = g.insert_sexpr(sexp);
        let ruleset = &read_ruleset(format!("src/rulesets/rulesetA.txt"));
        g.rewrite_ruleset(ruleset);
        g.rebuild();
        g.rewrite_ruleset(ruleset);
        g.rebuild();
        g.rewrite_ruleset(ruleset);
        g.rebuild();
        g.print();
        
        if let Some(cls) = g.get_eclass_cpy(root_id){
            for n in cls.nodes{
                print!("res: {}\n", extract_random(&g, n));
            }
        }
    }

    use rand::{thread_rng, Rng};
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

    const INTS: &str = "ints/";
    const IEXAMPLE: &str = "example.txt";

    const PEANO: &str = "peano/";
    const PSUM: &str = "sum.txt";
    const ZEROS: &str = "add_zeros.txt";

    //rulesets
    const R_PEANO: &str = "src/rulesets/peano_ruleset.txt";
    const R_A: &str = "src/rulesets/rulesetA.txt";
    const R_ZEROS: &str = "src/rulesets/recursive_rule.txt"; 


    #[test] //extracts the best found term from a set of options
    fn term_extraction(){
        let filepath = format!("{PATH}{INTS}{ZEROS}");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        let root_id = g.insert_sexpr(sexp);
        let ruleset = &read_ruleset(format!("{R_ZEROS}"));

        for i in 0..2{
            print!("rewrite {}\n", i);
            g.rewrite_ruleset(ruleset);
        }
        g.print();
        

        if let Some(str) =  g.extract_shortest(root_id){
            if let Ok(res) = parser::parse_str(&str){
                pretty_print(&res, 10);
            }
        } else{
            print!("\nFailure to find extractable sexpr\n");
        }
    }
}
