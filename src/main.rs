#![allow(unused)] //DISABLE SHITTY CODE WARNS


use std::{env::Vars, fmt::Debug};



pub(crate) type BuildHasher = fxhash::FxBuildHasher;
pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;
pub(crate) type IndexSet<K> = indexmap::IndexSet<K, BuildHasher>;
mod util;
use util::pretty_print;
use symbolic_expressions::{Sexp, SexpError, parser};
mod unionfind;
use unionfind::UnionFind;


#[macro_export]
macro_rules! mstr { //simplify string::from function
    ( $($x:expr)? ) =>{
        $( String::from($x) )+
    };
}


#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id(u32);
impl From<usize> for Id { //cast usize to Id
    fn from(n: usize) -> Id {
        Id(n as u32)
    }
}
impl From<Id> for usize { //cast Id to usize
    fn from(id: Id) -> usize {
        id.0 as usize
    }
}



#[derive(Clone)]
#[cfg_attr(feature = "serde-1", derive(Serialize, Deserialize))]
pub struct EGraph<L: Language, N: Analysis<L>> {
    /// The `Analysis` given when creating this `EGraph`.
    pub analysis: N,
    /// The `Explain` used to explain equivalences in this `EGraph`.
    pub(crate) explain: Option<Explain<L>>,
    unionfind: UnionFind,
    /// Stores the original node represented by each non-canonical id
    nodes: Vec<L>,
    /// Stores each enode's `Id`, not the `Id` of the eclass.
    /// Enodes in the memo are canonicalized at each rebuild, but after rebuilding new
    /// unions can cause them to become out of date.
    memo: HashMap<L, Id>,
    /// Nodes which need to be processed for rebuilding. The `Id` is the `Id` of the enode,
    /// not the canonical id of the eclass.
    pending: Vec<Id>,
    analysis_pending: UniqueQueue<Id>,
    pub(crate) classes: HashMap<Id, EClass<L, N::Data>>,
    pub(crate) classes_by_op: HashMap<L::Discriminant, HashSet<Id>>,
    /// Whether or not reading operation are allowed on this e-graph.
    /// Mutating operations will set this to `false`, and
    /// [`EGraph::rebuild`] will set it to true.
    /// Reading operations require this to be `true`.
    /// Only manually set it if you know what you're doing.
    pub clean: bool,
}




fn main() {
    let path = "src/testsuite/";
    let filename = "ints/nested_add.txt";


    use util;

    let buf = format!("{path}{filename}");
    let r = parser::parse_file(&buf).unwrap();

    pretty_print(&r, 10);

}
