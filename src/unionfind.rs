use crate::Id;
use std::fmt::Debug;
use std::vec::Vec;

//mostly originates from the egg library
//required for the Egraphs themselves

#[derive(Debug, Clone, Default)]
pub struct UnionFind {
    parents: Vec<Id>,
}

//all functions of the UnionFind struct
impl UnionFind {
    pub fn new_set(&mut self) -> Id {
        let id = Id::from(self.parents.len());
        self.parents.push(id);
        return id;
    }

    //get own size
    pub fn size(&self) -> usize {
        return self.parents.len();
    }

    //returns whatever ID the querried item points at
    fn parent(&self, query: Id) -> Id {
        return self.parents[usize::from(query)];
    }

    //same as non_mut but lets you edit the ID
    fn parent_mut(&mut self, query: Id) -> &mut Id {
        &mut self.parents[usize::from(query)]
    }

    //finds the canonical ID without editing the graph
    pub fn find(&self, mut current: Id) -> Id {
        while current != self.parent(current) {
            current = self.parent(current)
        }
        return current;
    }

    // finds the canonical ID, then sets all nodes in the set to point directly towards the canonical ID
    pub fn find_mut(&mut self, mut current: Id) -> Id {
        while current != self.parent(current) {
            let grandparent = self.parent(self.parent(current));
            *self.parent_mut(current) = grandparent;
            current = grandparent;
        }
        return current;
    }

    /// unions 2 nodes and returns root(id1)
    pub fn union(&mut self, id1: Id, id2: Id) -> Id {
        let root1 = self.find(id1);
        let root2 = self.find(id2);
        *self.parent_mut(root2) = root1;
        return root1;
    }

    //checks if 2 nodes are in the same set
    pub fn in_same_set(&self, id1:Id, id2:Id) -> bool {
        return self.find(id1) == self.find(id2);
    }

}


