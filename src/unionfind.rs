use crate::Id;
use std::fmt::Debug;
use std::vec::Vec;



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
    // for example in UF[0,3,0,2,4] find(1) loops from id==1 to id==3 to id==2 to id==0
    // then UF is set to [0,0,0,0,4]
    pub fn find_mut(&mut self, mut current: Id) -> Id {
        while current != self.parent(current) {
            let grandparent = self.parent(self.parent(current));
            *self.parent_mut(current) = grandparent;
            current = grandparent;
        }
        return current;
    }

    /// unions 2 nodes
    pub fn union(&mut self, id1: Id, id2: Id) -> Id {
        let root1 = self.find(id1);
        let root2 = self.find(id2);
        *self.parent_mut(root2) = root1;
        return root1;
    }

    //checks if 2 nodes are in the same set
    pub fn in_same_set(&mut self, id1:Id, id2:Id) -> bool {
        return self.find(id1) == self.find(id2);
    }

}


#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope


    #[test]
    fn union_find() { //run this test function to get a bit of insight how the unionfind works
        let n = 10;
        let id = Id::from;

        let mut uf = UnionFind::default();
        print!("base set:                   {:?}\n", uf);
        for _ in 0..n {
            uf.new_set();
            
        }

        // test the initial condition of everyone in their own set
        print!("after adding some nodes:    {:?}\n", uf);

        // build up one set
        uf.union(id(0), id(1));
        uf.union(id(0), id(2));
        uf.union(id(0), id(3));

        print!("after making 1 set:         {:?}\n", uf);

        // build up another set
        uf.union(id(6), id(7));
        uf.union(id(6), id(8));
        uf.union(id(6), id(9));

        print!("after making 1 more set:    {:?}\n", uf);

        uf.union(id(2), id(9));

        print!("after union:                {:?}\n", uf);

        // this compresses all paths
        for i in 0..n {
            uf.find_mut(id(i));
        }
        
        print!("after compression:          {:?}\n", uf);

        // indexes:                   0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        let expected = vec![0, 0, 0, 0, 4, 5, 0, 0, 0, 0];
    }
}