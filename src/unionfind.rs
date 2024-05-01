use crate::Id;
use std::fmt::Debug;
use std::vec::Vec;



#[derive(Debug, Clone, Default)]
pub struct UnionFind {
    parents: Vec<Id>,
}

//all functions of the UnionFind struct
impl UnionFind {
    pub fn make_set(&mut self) -> Id {
        let id = Id::from(self.parents.len());
        self.parents.push(id);
        id
    }

    //get own size
    pub fn size(&self) -> usize {
        self.parents.len()
    }

    //returns whatever ID the querried item points at
    fn parent(&self, query: Id) -> Id {
        self.parents[(query.0 as usize)]
    }

    //same as non_mut but lets you edit the ID
    fn parent_mut(&mut self, query: Id) -> &mut Id {
        &mut self.parents[query.0 as usize]
    }

    //finds the canonical ID, for example in UF[0,3,0,2] find(1) loops from id==1 to id==3 to id==2 to id==0
    pub fn find(&self, mut current: Id) -> Id {
        while current != self.parent(current) {
            current = self.parent(current)
        }
        current
    }

    //finds the canonical ID, then sets all nodes in the set to point directly towards the canonical ID
    //(e.g the previous example UF[0,3,0,2] would be set to UF[0,0,0,0])
    pub fn find_mut(&mut self, mut current: Id) -> Id {
        while current != self.parent(current) {
            let grandparent = self.parent(self.parent(current));
            *self.parent_mut(current) = grandparent;
            current = grandparent;
        }
        current
    }

    /// Given two leader ids, unions the two eclasses making root1 the leader.
    /// if unions are failing, check if the input IDs are the canonical IDs
    pub fn union(&mut self, root1: Id, root2: Id) -> Id {
        *self.parent_mut(root2) = root1;
        root1
    }
}

#[cfg(test)]
mod tests {
    use super::*; //allows this module to use previous scope

    fn ids(us: impl IntoIterator<Item = usize>) -> Vec<Id> {
        us.into_iter().map(|u| u.into()).collect()
    }

    #[test]
    fn union_find() {
        let n = 10;
        let id = Id::from;

        let mut uf = UnionFind::default();
        print!("base set:                   {:?}\n", uf);
        for _ in 0..n {
            uf.make_set();
        }

        // test the initial condition of everyone in their own set
        assert_eq!(uf.parents, ids(0..n));
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

        uf.union(uf.find(id(0)), uf.find(id(9)));

        print!("after union:                {:?}\n", uf);

        // this compresses all paths
        for i in 0..n {
            uf.find_mut(id(i));
        }
        
        print!("after compression:          {:?}\n", uf);

        // indexes:                   0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        let expected = vec![0, 0, 0, 0, 4, 5, 0, 0, 0, 0];
        assert_eq!(uf.parents, ids(expected));
    }
}