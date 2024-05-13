use indexmap::IndexSet;
use crate::Id;
use crate::Term;




#[derive(Clone, Debug)]
pub struct EClass {
    pub nodes: Vec<Term>, //Nodes part of this Eclass
    pub parents: IndexSet<Id>  //Parent Eclasses that point towards this Eclass
}
impl EClass {
    pub fn new(node: Term) -> EClass{
        let mut termvec = Vec::<Term>::new();
        termvec.push(node);
        let res = EClass {
            nodes: termvec,
            parents: IndexSet::<Id>::default()
        };
        return res;
    }
}