#![allow(unused)] //DISABLE SHITTY CODE WARNS
//note: this code uses "return val;" instead of "val" to make it clearer what is happening




mod id; use id::Id;
mod util;
use symbolic_expressions::{Sexp, parser};
use egraph::EGraph;
use pattern::read_ruleset;

mod enode;
use enode::Enode;

mod unionfind;
use unionfind::UnionFind;

mod eclass;
use eclass::EClass;

mod egraph;


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

//run these tests on your local machine
#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope

    use symbolic_expressions::{Sexp, parser};
    use egraph::EGraph;
    static PATH: &str = "src/testsuite/";


}


