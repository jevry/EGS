#![allow(unused)] //DISABLE SHITTY CODE WARNS
//note: this code uses "return val;" instead of "val" to make it clearer what is happening



use hashbrown::HashMap;

mod id; use id::Id;
mod util;


use util::pretty_print;




mod enode;
use enode::Enode;

mod unionfind;
use unionfind::UnionFind;

mod eclass;
use eclass::EClass;

mod egraph;
use egraph::EGraph;

mod pattern;






use symbolic_expressions::parser::parse_str;
use pattern::Rule;



//TODO: empty this defunct code
fn main() {
}


//run these tests on your local machine
#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope
    use symbolic_expressions::{Sexp, parser};
    static PATH: &str = "src/testsuite/";

    #[test] //to test rewriting a graph multiple times
    pub fn egraph_mass_rewrite() {
        let filepath = format!("{PATH}ints/example.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();

        let sexp1 = parse_str("(* P_x 2)").unwrap();
        let sexp2 = parse_str("(<< P_x 1)").unwrap();
        let r1 = Rule::new_rule(sexp1, sexp2).unwrap();

        let sexp1 = parse_str("(* P_x P_y)").unwrap();
        let sexp2 = parse_str("(* P_y P_x)").unwrap();
        let r2 = Rule::new_rule(sexp1, sexp2).unwrap();

        let sexp1 = parse_str("(* (P_x P_y) P_z)").unwrap();
        let sexp2 = parse_str("(* P_x (P_y P_z))").unwrap();
        let r3 = Rule::new_rule(sexp1.clone(), sexp2.clone()).unwrap();
        let r4 = Rule::new_rule(sexp2, sexp1).unwrap();

        let sexp1 = parse_str("(/ P_c P_c)").unwrap();
        let sexp2 = parse_str("1").unwrap();
        let r5 = Rule::new_rule(sexp1.clone(), sexp2.clone()).unwrap();

        let sexp1 = parse_str("(/ (* P_x P_y) P_z)").unwrap();
        let sexp2 = parse_str("(* P_x (/ P_y P_z))").unwrap();
        let r6 = Rule::new_rule(sexp1.clone(), sexp2.clone()).unwrap();

        let ruleset = [r1, r2, r3, r4, r5, r6].to_vec();
        let edits = g.rewrite_ruleset(ruleset.clone());
        print!("first pass edits: {}\n", edits);
        let edits = g.rewrite_ruleset(ruleset.clone());
        print!("second pass edits: {}\n", edits);
        let edits = g.rewrite_ruleset(ruleset);
        print!("third pass edits made: {}\n", edits);
        g.print();
    }

}


