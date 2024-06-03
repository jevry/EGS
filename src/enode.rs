
use crate::id::Id;

//the terms we use inside of the egraph, currently only supports Symbolic expressions
//though in theory it can support other data as well.


#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Enode {
    pub head: String,
    pub args: Vec<Id>
}
impl Enode {
    pub fn new(string: String) -> Enode {
        let test = Enode  {
            head: string.clone(),
            args: Vec::new()
        };
        return test;
    }
}