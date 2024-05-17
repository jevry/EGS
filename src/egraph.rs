use std::clone;
use std::thread::yield_now;
use std::vec;


use symbolic_expressions::{Sexp, SexpError, parser};
use crate::Id;//the structure
use crate::itoid;//the macro
use crate::Term;
use crate::UnionFind;
use crate::EClass;
use crate::mstr;
use crate::Pattern;
use indexmap::{map::Values, IndexSet};


//we use hasmap instead of indexmap because indexmaps are more deterministic and easier to debug
pub(crate) type BuildHasher = fxhash::FxBuildHasher;
pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;


//   `memo` to map `Term` to their equivalence class ID
//`classes` to map equivalence class `Id` to the `EClass`
#[derive(Debug, Clone)]
pub struct EGraph{
    unionfind:    UnionFind,
    memo:         IndexMap<Term, Id>,  //memory to store future term IDs
    classes:      IndexMap<Id,EClass>,
    dirty_unions: Vec<Id>,
}
impl EGraph{
    // Build an empty EGraph
    pub fn new() -> EGraph{
        let g = EGraph{
            unionfind:    UnionFind::default(),
            memo:         IndexMap::<Term, Id>::default(),
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
    pub fn get_class(&self, id: Id) -> Option<&EClass>{
        return self.classes.get(&id);
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
    fn match_pattern(&mut self ,e:EClass,  p:&Pattern, sub: &mut IndexMap<Term, Id>) -> Option<IndexMap<Pattern, Term>> {
        if let Pattern::PatVar(s) = p {
            let mut m = IndexMap::<Pattern, Term>::default();

            for t in e.nodes{
                m.insert(Pattern::PatVar(s.clone()), t.clone()); //probably 1
                let id = self.find_eclass(&mut t.clone()).unwrap();
                print!("match pattern putting into buf: {:?}\n", t.clone());
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
                    let mut res = Vec::<IndexMap<Pattern, Term>>::new();
                    for (t1,p1) in t.args.iter().zip(p_args){ //for each arg in the term
                        let c = self.get_class(*t1).unwrap(); //derive eclass from t.arg
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

    fn merge_consistent(dicts: Vec<IndexMap<Pattern, Term>>) -> Option<IndexMap<Pattern, Term>>{
        let mut newd: IndexMap<Pattern, Term> = IndexMap::<Pattern, Term>::default();
    
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
    fn instantiate(&mut self, p: Pattern, sub: &IndexMap<Term, Id>) -> Option<Id>{
        if let Pattern::PatVar(s) = p { //adding a patvar to the EG
            print!("pattern head = {}\n", mstr!(s.clone()));
            print!("instantiate received {:?}\n", sub);
            let t = sub.get(&Term::new(s)); //get the appropriate ID
            if t.is_none(){print!("PANIC!!!\n")}
            return Some(*t.unwrap()); //return the appropriate ID

        } else if let Pattern::PatTerm(p_head, p_args) = p {
            let mut t = Term::new(p_head); //get term head
            for a in p_args { //get terms in p_args

                //recursive call, return val is the ID of the Enode the call added to the EGraph
                t.args.push(self.instantiate(*a, &sub).unwrap());//push that ID to our term
            }
            return Some(self.push_eclass(&mut t.clone())); //push term into EGraph
        }
        return None; //instantiate breaks
    }
}



impl EGraph{
    //returns the canonical id
    pub fn find_root(&mut self, id:Id)-> Id{
        return self.unionfind.find_mut(id);
    }

    // Checks if 2 given terms are in the same class.
    // Panics if either of the terms aren't present in the egraph.
    pub fn in_same_class(&mut self, t1:Term, t2:Term) -> bool{
        let (_, id1) = self.memo.get_key_value(&t1).unwrap();
        let (_, id2) = self.memo.get_key_value(&t2).unwrap();
        return self.unionfind.in_same_set(*id1, *id2);
    }

    // canonicalizes the args of a given term, then returns it
    pub fn canonicalize(&mut self, term: &mut Term){
        let mut new = Vec::<Id>::new();
        for i in term.args.clone(){
            new.push(self.unionfind.find_mut(i));
        }
        term.args = new;
    }

    //finds the corresponding Eclass Id of a given term
    //returns None if it cant find the Eclass
    pub fn find_eclass(&mut self, term: &mut Term) -> Option<Id>{
        self.canonicalize(term);
        let pair = self.memo.get_key_value(term);
        if pair.is_none(){
            return None;
        }
        else{
            let (_, id) = pair.unwrap();
            return Some(self.find_root(*id));
        }
    }

    // Push a potentially new eclass to the graph, then return Id
    pub fn push_eclass(&mut self, term:&mut Term) -> Id{
        let id = self.find_eclass(term);
        if id.is_some(){ //term already in the graph
            return id.unwrap();
        }
        let id = self.unionfind.new_set();
        let eclass = EClass::new(term.clone());


        self.memo.insert(term.clone(), id);
        for child in term.args.clone(){ // set parent pointers
            self.classes[usize::from(child)].parents.insert(id);
        }
        self.classes.insert(id, eclass);
        return id;
    }

    //unions 2 eclasses and returns the new canonical Id
    //returns None if the 2 classes are already in the same class
    pub fn union(&mut self, id1: Id, id2: Id) -> Option<Id> {
        let (id1, id2) = (self.find_root(id1), self.find_root(id2));
        if id1 == id2{ return None }
        
        let id3 = self.unionfind.union(id1,id2); 
        self.dirty_unions.push(id3); // id3 will need it's parents processed in rebuild!

        let (to_id, from_id) = (id1, id2);
        let mut from = self.classes.get(&id2).unwrap().clone();
        let mut to = self.classes.get(&id1).unwrap().clone();

        // we empty out the eclass[from] and put everything in eclass[to]
        to.nodes.extend(from.nodes);
        to.parents.extend(from.parents);

        // recanonize all nodes in memo.
        for t in &mut to.nodes{
            let tid = self.memo.get(t).unwrap().clone();
            self.memo.swap_remove(t);
            self.canonicalize(t);
            self.memo.insert(t.clone(), tid);
        }

        self.classes.insert(to_id, to);

        self.classes.swap_remove(&from_id);

        return Some(id3);
    }

    // Push a potentially new const eclass to the graph, then return Id
    pub fn constant(&mut self, x: String) -> Id{
        let mut t = Term::new(x);
        return self.push_eclass(&mut t);
    }

    //insert a new Sexpr into the Egraph
    //this does so using recursion and merges already existing terms.
    pub fn term(&mut self, f: Sexp) -> Id{
        let mut term: Term;
        if let Sexp::List(list) = f { //sexp is operator
            let op = mstr!(list[0].to_string().trim_matches('"'));
            term = Term::new(op);
            for item in &list[1..]{//process the args
                let id = self.term(item.clone());
                term.args.push(id);
            }
            return self.push_eclass(&mut term);
        } else { //sexp is leaflet
            let s = mstr!(f.to_string().trim_matches('"'));
            return self.constant(s);
        }
    }
}

struct Rule {
    lhs: Pattern,
    rhs: Pattern
}


impl EGraph {
    fn rewrite(&mut self, r:Rule){
        let mut matches = Vec::<(Id, Id)>::new();

        let mut bufdict = IndexMap::<Term, Id>::default();
        let t = r.lhs.clone();
        for (n, cls) in self.classes.clone(){
            let temp =  self.match_pattern(cls, &t, &mut bufdict);
            if temp.is_some() {
                    print!("found match {:?} for id {:?}\n", temp, n);
                    for sub in temp.unwrap(){
                        print!("insering the term...\n");

                        // matches.push(( self.instantiate(r.lhs ,sub), instantiate(r.rhs ,sub) )); 
                    }



                    self.instantiate(t.clone(), &bufdict);
                    print!("success!\n\n")

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
        g.term(sexp);

        print!("\nnew graph: ");
        g.print();
    }

    #[test] //run this test function to see adding a new term to a constructed graph
    fn egraph_editing() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.term(sexp);
        print!("\nnew egraph: ");
        g.print();

        let altsexp: Sexp = parser::parse_str("(<< a 1)").unwrap();

        print!("\nextra term: {:?}\n", altsexp);
        
        g.term(altsexp);

        print!("\nedited graph: ");
        g.print();
    }

    #[test] //run this test function to see adding a new term to a constructed graph
    fn egraph_union() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.term(sexp);
        let altsexp: Sexp = parser::parse_str("(<< a 1)").unwrap();
        g.term(altsexp);

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
        g.term(sexp);
        print!("\nnew egraph: ");
        g.print();

        let mut vec = Vec::<Box<Pattern>>::new();
        let patv = Pattern::PatVar(mstr!("P_a"));
        vec.push(Box::<Pattern>::new(patv));
        let patv = Pattern::PatVar(mstr!("P_b"));
        vec.push(Box::<Pattern>::new(patv));
        let patt = Pattern::PatTerm(mstr!("*"), vec);
        print!("{:?}\n", patt);

        let mut vec2 = Vec::<Box<Pattern>>::new();
        let patv2 = Pattern::PatVar(mstr!("P_a"));
        vec2.push(Box::<Pattern>::new(patv2));
        let patv2 = Pattern::PatTerm(mstr!("P_b"), Vec::<Box<Pattern>>::new());
        vec2.push(Box::<Pattern>::new(patv2));
        let patt2 = Pattern::PatTerm(mstr!("*"), vec2);
        print!("{:?}\n", patt2);

        let n = g.get_class(itoid!(2)).unwrap();
        let mut dict = IndexMap::<Term, Id>::default();
        print!("\nres = {:?}\n", g.match_pattern(n.clone(), &patt.clone(), &mut dict));
        print!("var map = {:?}\n\n", dict);

        let r = Rule {lhs: patt, rhs:patt2}; 
        g.rewrite(r);
    }
}
