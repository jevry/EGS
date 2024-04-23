#![allow(unused)] //DISABLE SHITTY WARNS

//use std::io;

pub struct TreeNode {
    pub child: Option<Box<TreeNode>>,
    pub strn: String,
}


pub fn build_node(string: String) -> TreeNode {
    TreeNode {
        child: None,
        strn: string
    }
}

