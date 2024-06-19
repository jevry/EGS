/*
 * main.rs
 * -------------------------
 * Author  : Kieran van Gelder
 * Id      : 14033623
 * Date    : 2024
 * Version : 0.1
 * -------------------------
 * The main file, used as a import test and showcase.
 * lib.rs is the "main" file that collects and reexports the other files.
 * this file can be safely ignored
 * 
 * Some rustic things that might be usefull to know:
 * - use /// to indicate that a comment is outer line doc
 * - 'return n' can be rewritten as 'n', though
 * for clarity sake this isnt used in this library
 * - #[derive()] auto generates certain functionality for structs and enums
 * - in lib.rs are some test functions, you can run these in vsc or from the terminal
 */

use egs;
use egs::{Sexp, egraph::EGraph, parser};

//a carbon copy of the extract_example function in the tests under lib.rs
fn main() {
    static PATH: &str = "src/testsuite/";

    let filepath = &format!("{PATH}ints/example.txt");
    let rulepath = &format!("src/rulesets/rulesetA.txt");
    let iterations = 3;
    rewrite_extract(filepath, rulepath, iterations);
}

/// convert file into a egraph, rewrite it n times according to the rulefile
/// a extract the best candidate using extract_logical
pub fn rewrite_extract(filepath: &str, rulepath: &str, n: u32){
    let sexp: Sexp = parser::parse_file(filepath).unwrap();
    let mut g = EGraph::new();
    let root_id = g.insert_sexpr(sexp);
    let ruleset = &egs::read_ruleset(rulepath);

    for i in 0..n{
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