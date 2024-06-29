/*
* eclass.rs
* -------------------------
* Author  : Kieran van Gelder
* Id      : 14033623
* Date    : 2024
* Version : 0.1
* -------------------------
* the eclass struct
* 
*/
use indexmap::IndexSet;
use crate::Id;
use crate::Enode;

///the eclass struct itself
#[derive(Clone, Debug)]
pub struct EClass {
    //Nodes part of this Eclass, indexset is usefull because duplicates are useless and break congruence verification
    pub nodes: IndexSet<Enode>,

    //Parent Eclasses that point towards this Eclass
    pub parents: Vec::<(Enode, Id)>
}

impl EClass {
    pub fn new(node: Enode) -> EClass{
        let mut termvec = IndexSet::<Enode>::new();
        termvec.insert(node);
        let res = EClass {
            nodes: termvec,
            parents: Vec::<(Enode, Id)>::new()
        };
        return res;
    }
}