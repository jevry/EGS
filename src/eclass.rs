use crate::Id;
use crate::Enode;




#[derive(Clone, Debug)]
pub struct EClass {
    pub nodes: Vec<Enode>, //Nodes part of this Eclass
    pub parents: Vec::<(Enode, Id)>  //Parent Eclasses that point towards this Eclass
}
impl EClass {
    pub fn new(node: Enode) -> EClass{
        let mut termvec = Vec::<Enode>::new();
        termvec.push(node);
        let res = EClass {
            nodes: termvec,
            parents: Vec::<(Enode, Id)>::new()
        };
        return res;
    }

    pub fn empty() -> EClass{
        let mut termvec = Vec::<Enode>::new();
        let res = EClass {
            nodes: termvec,
            parents: Vec::<(Enode, Id)>::new()
        };
        return res;
    }
}