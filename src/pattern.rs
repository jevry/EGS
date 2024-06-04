//pattern
//used for pattern matching

use std::fs::read_to_string;
use symbolic_expressions::parser::parse_str;
use symbolic_expressions::{Sexp, SexpError};
use crate::mstr;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Pattern {
    PatVar(String),
    PatTerm(String, Vec<Box<Pattern>>),
}

#[derive(Debug, Clone)]
pub(crate) struct Rule {
    pub lhs: Pattern,
    pub rhs: Pattern
}

//build a new pattern from a given Sexp
//variables must start with P_ or they will be taken as consts
pub fn new_pattern(sexpr: Sexp) -> Option<Pattern>{
    if let Ok(l) = sexpr.list(){
        let mut vec = Vec::<Box<Pattern>>::new();
        for i in &l[1..]{
            let arg = new_pattern(i.clone()).unwrap();
            vec.push(Box::<Pattern>::new(arg));
        }
        if let Ok(str) = l[0].string() {
            let patt = Pattern::PatTerm(str.clone(), vec);
            return Some(patt);
        }
        return None;
    } else if let Ok(s) = sexpr.string(){
        if s.starts_with("P_"){
            let patt = Pattern::PatVar(mstr!(s));
            return Some(patt);
        } else {
            let patt = Pattern::PatTerm(mstr!(s), Vec::<Box<Pattern>>::new());
            return Some(patt);
        }
    }
    return None;
}
impl Rule{
    pub fn new_rule(lhs: Sexp, rhs: Sexp) -> Option<Rule>{
        if let Some(lhs) = new_pattern(lhs){
            if let Some(rhs) = new_pattern(rhs){
                let r = Rule{lhs, rhs};
                return Some(r);
            }
        }
        return None;
    }
}


pub fn read_ruleset(filepath: String) -> Vec::<Rule> {
    let mut res = Vec::<Rule>::new();
    for line in read_to_string(filepath).unwrap().lines() {
        if line.len() == 0{continue;}
        let parts = line.split("->").collect::<Vec<&str>>();
        if parts.len() > 1{
            if let Ok(lhs) = parse_str(parts[0]){
                if let Ok(rhs) = parse_str(parts[1]){
                    let r = Rule::new_rule(lhs, rhs).unwrap();
                    res.push(r);
                }
            }
        }
    }
    return res;
}

//run these tests on your local machine
#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope
    use std::fs::read_to_string;
    use symbolic_expressions::parser::parse_str;

    static PATH: &str = "src/rulesets/";
    static FILENAME: &str = "patternB.txt";

    #[test] //run this test function to see graph construction
    fn construct_rule(){
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
}