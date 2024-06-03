

use symbolic_expressions::{Sexp, SexpError, parser};
use crate::{Id, itoid};
use crate::Enode;
use crate::UnionFind;
use crate::EClass;
use crate::mstr;
use crate::pattern::{Pattern, Rule};
use indexmap::{map::Values, IndexSet};


//we use hasmap instead of indexmap because indexmaps are more deterministic and easier to debug
pub(crate) type BuildHasher = fxhash::FxBuildHasher;
pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;


//   `memo` to map `Term` to their equivalence class ID
//`classes` to map equivalence class `Id` to the `EClass`
#[derive(Debug, Clone)]
pub struct EGraph{
    unionfind:    UnionFind,
    memo:         IndexMap<Enode, Id>,  //memory to store future term IDs
    classes:      IndexMap<Id,EClass>,
    dirty_unions: Vec<Id>,
}
impl EGraph{
    pub fn new() -> EGraph{
        let g = EGraph{
            unionfind:    UnionFind::default(),
            memo:         IndexMap::<Enode, Id>::default(),
            classes:      IndexMap::<Id,EClass>::default(),
            dirty_unions: Vec::<Id>::new()
        };
        return g;
    }

    pub fn print(&self){
        print!("{:?}\n", self.unionfind);
        print!("memo\n");
        for t in self.memo.clone(){
            print!("{:?} \n", t);
        }
        print!("\nclasses\n");
        for t in self.classes.clone(){
            print!("{:?} \n", t);
        }
        print!("dirty_unions {:?}\n", self.dirty_unions);
    }

    //return a eclass of a given id
    pub fn get_eclass(&self, id: Id) -> Option<&EClass>{
        return self.classes.get(&id);
    }


}

//basic egraph manipulation
impl EGraph{
    //returns the canonical id
    pub fn find(&mut self, id:Id)-> Id{
        return self.unionfind.find_mut(id);
    }

    // Checks if 2 given terms are in the same class.
    // Panics if either of the terms aren't present in the entire egraph.
    pub fn in_same_class(&mut self, t1:Enode, t2:Enode) -> bool{
        let (_, id1) = self.memo.get_key_value(&t1).unwrap();
        let (_, id2) = self.memo.get_key_value(&t2).unwrap();
        return self.unionfind.in_same_set(*id1, *id2);
    }

    // canonicalizes the args of a given term, then returns it
    pub fn canonicalize_args(&mut self, term: &mut Enode){
        let mut new = Vec::<Id>::new();
        for i in term.args.clone(){
            new.push(self.unionfind.find_mut(i));
        }
        term.args = new;
    }

    //finds the corresponding Eclass Id of a given enode
    //returns None if it cant find the Eclass
    pub fn find_eclass(&mut self, enode: &mut Enode) -> Option<Id>{
        self.canonicalize_args(enode);
        let pair = self.memo.get_key_value(enode);
        if pair.is_none(){
            return None;
        }
        else{
            let (_, id) = pair.unwrap();
            return Some(self.find(*id));
        }
    }

    // Push a potentially new eclass to the graph, then return Id
    pub fn push_eclass(&mut self, enode:&mut Enode) -> Id{
        let id = self.find_eclass(enode);
        if id.is_some(){ //term already in the graph
            return id.unwrap();
        }
        let id = self.unionfind.new_set();
        let eclass = EClass::new(enode.clone());


        self.memo.insert(enode.clone(), id);
        for child in enode.args.clone(){ // set parent pointers
            self.classes[usize::from(child)].parents.insert(id);
        }
        self.classes.insert(id, eclass);
        return id;
    }

    //unions 2 eclasses and returns the new canonical Id
    //returns None if the 2 classes are already in the same class
    pub fn union(&mut self, id1: Id, id2: Id) -> Option<Id> {
        let (id1, id2) = (self.find(id1), self.find(id2));
        if id1 == id2{ return None }
        
        let id3 = self.unionfind.union(id1,id2); 
        self.dirty_unions.push(id3); // id3 will need it's parents processed in rebuild!

        let (to_id, from_id) = (id1, id2);
        let mut from = self.classes.get(&id2).unwrap().clone();
        let mut to = self.classes.get(&id1).unwrap().clone();

        // move nodes and parents from eclass[`from`] to eclass[`to`]
        to.nodes.extend(from.nodes);
        to.parents.extend(from.parents);

        // recanonize all nodes in memo.
        for t in &mut to.nodes{
            let tid = self.memo.get(t).unwrap().clone();
            self.memo.swap_remove(t);
            self.canonicalize_args(t);
            self.memo.insert(t.clone(), tid);
        }

        self.classes.insert(to_id, to); //replace old `to` with new `to`
        self.classes.swap_remove(&from_id); //remove old `from`

        return Some(id3);
    }

    // Push a potentially new const eclass to the graph, then return Id
    pub fn constant(&mut self, x: String) -> Id{
        let mut t = Enode::new(x);
        return self.push_eclass(&mut t);
    }

    //insert a new Sexpr into the Egraph
    //this does so using recursion and merges already existing terms.
    pub fn insert_sexpr(&mut self, f: Sexp) -> Id{
        let mut term: Enode;
        if let Sexp::List(list) = f { //sexp is operator
            let op = mstr!(list[0].to_string().trim_matches('"'));
            term = Enode::new(op);
            for item in &list[1..]{//process the args
                let id = self.insert_sexpr(item.clone());
                term.args.push(id);
            }
            return self.push_eclass(&mut term);
        } else { //sexp is leaflet
            let s = mstr!(f.to_string().trim_matches('"'));
            return self.constant(s);
        }
    }
}

/*
impl EGraph{
    fn ematchlist(self, t:Vec<Box<Pattern>> , v:Vec<Id>, sub: IndexMap<Pattern, Id>) -> IndexMap<Pattern, Id>{
        //Channel() do c
            if t.len() == 0{
                return sub;
            }else{
                let it = self.ematch(*t[0], v[0], sub);
                t.remove(0);
                v.remove(0);
                for sub1 in it{
                    let res = self.ematchlist(t, v, sub1);
                    return res;

                }
            }
        //end
        return sub;
    }

    // sub should be a map from pattern variables to Id
    fn ematch(&self, t:Pattern, v:Id, sub: IndexMap<Pattern, Id>) -> impl Iterator<Item = Pattern, Id> {
        let v_root = self.find_root(v);
        if let Pattern::PatVar(s) = t {
        //Channel() do c
            if sub.contains_key(&t){
                let temp = sub.get(&t).unwrap();
                if self.find_root(*temp) == v_root{
                    return sub;
                }
            } else {
                //return Base.ImmutableDict(sub, t => v_root); TODO
            }
        }
        else if let Pattern::PatTerm(t_head, t_args) = t {
        //Channel() do c
            let temp = *self.classes.get(&v_root).unwrap();
            for n in temp.nodes{
                if n.head == t_head{
                    for sub1 in self.ematchlist(t_args , n.args , sub){
                        return sub1;
                    }
                }
            }
        }
        return sub.values();
    }
    

}*/


//inefficient but simple pattern matching algorithm
impl EGraph{
    fn match_pattern(&mut self ,e:EClass,  p:&Pattern, sub: &mut IndexMap<Enode, Id>) -> Option<IndexMap<Pattern, Enode>> {
        if let Pattern::PatVar(s) = p {
            let mut m = IndexMap::<Pattern, Enode>::default();

            for t in e.nodes{
                m.insert(Pattern::PatVar(s.clone()), t.clone()); //probably 1
                let id = self.find_eclass(&mut t.clone()).unwrap();
                // print!("match pattern putting into buf: {:?}\n", t.clone());
                sub.insert(t.clone(), id);
            }
            return Some(m);
        }
        else if let Pattern::PatTerm(p_head, p_args) = p {
            for t in e.nodes{ //for each node in the eclass
                if t.head != *p_head || t.args.len() != p_args.len(){ //no match
                    continue;
                }
                else { //heads the same, check kids
                    let mut res = Vec::<IndexMap<Pattern, Enode>>::new();
                    for (t1,p1) in t.args.iter().zip(p_args){ //for each arg in the term
                        let c = self.get_eclass(*t1).unwrap(); //derive eclass from t.arg
                        let d = self.match_pattern(c.clone(),p1, sub);
                        if d.is_some(){ //if we had a result
                            res.push(d.unwrap());
                        }
                    }
                    return EGraph::merge_consistent( res );
                }
            }
        }
        return None;
    }

    fn merge_consistent(dicts: Vec<IndexMap<Pattern, Enode>>) -> Option<IndexMap<Pattern, Enode>>{
        let mut newd: IndexMap<Pattern, Enode> = IndexMap::<Pattern, Enode>::default();
    
        for dict in dicts {
            for (k,v) in dict{
                if newd.contains_key(&k){
                    if *newd.get(&k).unwrap() != v{
                        return None;
                    }
                }else{
                    newd.insert(k, v);
                }
            }
        }
        return Some(newd);
    }

    //sub maps a VAR symbol to an Identifier in the EGraph
    //this means that the VAR symbol comes from pattern and the ID from the EGraph
    fn instantiate(&mut self, p: Pattern, translator: &IndexMap<Pattern, Enode>, sub: &IndexMap<Enode, Id>) -> Option<Id>{
        if let Pattern::PatVar(s) = p { //adding a patvar to the EG
            
            let t = translator.get(&Pattern::PatVar(s.clone()));
            print!("instantiate received sub {:?}\n", sub);
            print!("instantiate received trns {:?}\n", translator);

            print!("pattern head = {}\n\n", mstr!(s));
            let tid = sub.get(t.unwrap()); //get the appropriate ID

            return Some(*tid.unwrap()); //return the appropriate ID

        } else if let Pattern::PatTerm(p_head, p_args) = p {
            let mut t = Enode::new(p_head); //get term head
            for a in p_args { //get terms in p_args

                //recursive call, return val is the ID of the Enode the call added to the EGraph
                t.args.push(self.instantiate(*a, translator,&sub).unwrap());//push that ID to our term
            }
            return Some(self.push_eclass(&mut t.clone())); //push term into EGraph
        }
        return None; //instantiate breaks
    }

    fn rewrite(&mut self, r:Rule){


        let mut bufdict = IndexMap::<Enode, Id>::default();
        let mut translator = IndexMap::<Pattern, Enode>::default();
        let lhs = r.lhs.clone();
        let rhs = r.rhs.clone();
        for (n, cls) in self.classes.clone(){
            let matches =  self.match_pattern(cls, &lhs, &mut bufdict);
            if matches.is_some() {
                    print!("found match {:?} for id {:?}\n", matches, n);
                    let temp = matches.unwrap();
                    // for sub in temp.clone(){
                    //     print!("insering the term...\n");
                    //     let id1 = self.instantiate(lhs.clone(), &temp,&bufdict);
                    //     let id2 = self.instantiate(rhs.clone(), &temp,&bufdict);
                    //     // matches.push(( self.instantiate(r.lhs ,sub), instantiate(r.rhs ,sub) )); 
                    // }



                    let id1 =  self.instantiate(lhs.clone(), &temp,&bufdict);
                    let id2 =  self.instantiate(rhs.clone(), &temp,&bufdict);
                    print!("success!\n\n");
                    self.union(id1.unwrap(), id2.unwrap());
            }
        }
        self.print();
    }
}


//run these tests on your local machine
#[cfg(test)]
mod tests {
    use symbol_table::Symbol;

    use super::*; //allows this module to use previous scope
    static PATH: &str = "src/testsuite/";
    static FILENAME: &str = "ints/nested_add.txt";

    #[test] //run this test function to see graph construction
    fn egraph_construction() {
        let filepath = format!("{PATH}{FILENAME}");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        print!("empty graph: {:?}\n", g);
        g.insert_sexpr(sexp);

        print!("\nnew graph: ");
        g.print();
    }

    #[test] //run this test function to see adding a new term to a constructed graph
    fn egraph_editing() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();

        let altsexp: Sexp = parser::parse_str("(<< a 1)").unwrap();

        print!("\nextra term: {:?}\n", altsexp);
        
        g.insert_sexpr(altsexp);

        print!("\nedited graph: ");
        g.print();
    }

    #[test] //run this test function to see adding a new term to a constructed graph
    fn egraph_union() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        let altsexp: Sexp = parser::parse_str("(<< a 1)").unwrap();
        g.insert_sexpr(altsexp);

        print!("\nedited graph: ");
        g.print();

        print!("\nunioning eclass 3 and 4...\n\n");
        g.union(itoid!(2), itoid!(4));
        g.print();
    }

    // pub enum Pattern {
    //     PatVar(String),
    //     PatTerm(String, Vec<Box<Pattern>>),
    // }


    #[test] //to test ematching
    fn egraph_matching() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();

        let mut vec = Vec::<Box<Pattern>>::new();
        let patv = Pattern::PatVar(mstr!("P_a"));
        vec.push(Box::<Pattern>::new(patv));
        let patv = Pattern::PatTerm(mstr!("2"), Vec::<Box<Pattern>>::new());
        vec.push(Box::<Pattern>::new(patv));
        let patt = Pattern::PatTerm(mstr!("*"), vec);
        print!("{:?}\n", patt);

        let mut vec2 = Vec::<Box<Pattern>>::new();
        let patv2 = Pattern::PatVar(mstr!("P_a"));
        vec2.push(Box::<Pattern>::new(patv2));
        let patv2 = Pattern::PatTerm(mstr!("1"), Vec::<Box<Pattern>>::new());
        vec2.push(Box::<Pattern>::new(patv2));
        let patt2 = Pattern::PatTerm(mstr!("<<"), vec2);
        print!("{:?}\n", patt2);

        let n = g.get_eclass(itoid!(2)).unwrap();
        let mut dict = IndexMap::<Enode, Id>::default();
        print!("\nres = {:?}\n", g.match_pattern(n.clone(), &patt.clone(), &mut dict));
        print!("var map = {:?}\n\n", dict);

        let r = Rule {lhs: patt, rhs:patt2}; 
        g.rewrite(r);
    }
}
