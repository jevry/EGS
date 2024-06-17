/*
 * main.rs
 * -------------------------
 * Author  : Kieran van Gelder
 * Id      : 14033623
 * Date    : 2024
 * Version : 0.1
 * -------------------------
 * The main file, used as a import test.
 * this file can be safely ignored
 * 
 */

use egs;
use egs::{Sexp, egraph::EGraph, parser};

//a carbon copy of the extract_example function in the tests under lib.rs
fn main() {
    static PATH: &str = "src/testsuite/";

    let filepath = &format!("{PATH}ints/example.txt");
    let rulepath = &format!("src/rulesets/rulesetA.txt");
    let iterations = 3;
    egs::rewrite_extract(filepath, rulepath, iterations);
}

pub fn rewrite_extract(filepath: &str, rulepath: &str, rewrites: u32){
    let sexp: Sexp = parser::parse_file(filepath).unwrap();
    let mut g = EGraph::new();
    let root_id = g.insert_sexpr(sexp);
    let ruleset = &egs::read_ruleset(rulepath);

    for i in 0..rewrites{
        print!("rewrite {}\n", i);
        g.rewrite_ruleset(ruleset);
    }
    print!("\n\n");

    g.print();
    

    if let Some(str) =  g.extract_logical(root_id){
        if let Ok(res) = parser::parse_str(&str){
            egs::pretty_print(&res, 10);
        }
    } else{
        print!("\nFailure to find extractable sexpr\n");
    }
}