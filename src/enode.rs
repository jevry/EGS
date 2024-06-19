/*
 * enode.rs
 * -------------------------
 * Author  : Kieran van Gelder
 * Id      : 14033623
 * Date    : 2024
 * Version : 0.1
 * -------------------------
 * the terms we use inside of the egraph, currently only supports Symbolic expressions
 * 
 */
use crate::id::Id;

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

impl Enode {
    ///number of args
    pub fn len(&self)-> usize {
        return self.args.len()
    }
}