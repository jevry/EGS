/*
 * egraph.rs
 * -------------------------
 * Author  : Kieran van Gelder
 * Id      : 14033623
 * Date    : 2024
 * Version : 0.1
 * -------------------------
 * Egraphs itself, the bulk of the code.
 * to improve readability the functions of
 * the egraph struct are seperated in 3 impl
 * blocks. the first covers; basic functionality,
 * unioning, rebuilding and term insertion.
 */

use symbolic_expressions::Sexp;
use crate::itoid;
use crate::Id;
use crate::Enode;
use crate::UnionFind;
use crate::EClass;
use crate::mstr;
use crate::pattern::{Pattern, Rule};
use indexmap::IndexSet;
use crate::util::has_unique_elements;

//we use indexmap instead of hashmap because indexmaps
//are more deterministic and easier to debug
pub(crate) type BuildHasher = fxhash::FxBuildHasher;
pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasher>;


//   `memo` to map `Term` to their equivalence class ID
//`classes` to map equivalence class `Id` to the `EClass`
#[derive(Debug, Clone)]
pub struct EGraph{
    pub(crate) unionfind:    UnionFind,
    pub(crate) memo:         IndexMap<Enode, Id>,  //memory to store future term IDs
    pub(crate) classes:      IndexMap<Id,EClass>,
    dirty_unions: Vec<Id>,
}
impl EGraph{
    /// create a new empty egraph
    pub fn new() -> EGraph{
        let g = EGraph{
            unionfind:    UnionFind::default(),
            memo:         IndexMap::<Enode, Id>::default(),
            classes:      IndexMap::<Id,EClass>::default(),
            dirty_unions: Vec::<Id>::new()
        };
        return g;
    }

    ///debugging: return a list of all enodes in the egraph
    fn enodes(&self) -> Vec<Enode>{
        let mut r = Vec::<Enode>::new();
        for (_, cls) in &self.classes{
            r.extend(&mut cls.nodes.clone().into_iter());
        }
        return r;
    }

    ///debugging: returns wheter the egraph is currently congruent or not
    ///returns false if not congruent
    pub fn congruent_invariant(&self) -> bool{
        let nodes = self.enodes();
        return has_unique_elements(nodes);
    }

    pub fn canonical_invariant(&self) -> bool{
        let nodes = self.enodes();
        for i in &nodes{
            let ci = self.canonicalize_args(&i);
            if ci != *i{
                return false
            }
        }
        return true
    }

    ///debugging: size of unionfind
    pub fn uf_len(&self) -> usize{
        return self.unionfind.len()
    }

    ///debugging: n of enodes
    pub fn n_enodes(&self) ->usize{
        let mut r = 0;
        for (_, cls) in &self.classes{
            r+=cls.nodes.len();
        }
        return r;
    }

    ///debugging: n of eclasses
    pub fn n_eclasses(&self) ->usize{
        return self.classes.len();
    }

    /// print out the egraph
    pub fn print(&self){
        print!("{:?}\n", self.unionfind);
        print!("memo\n");
        for t in &self.memo{
            print!("    {:?} \n", t);
        }
        print!("\nclasses\n");
        for (i, c) in &self.classes{
            print!("    eclass: {:?} \n", i);
            for t in &c.nodes{
                print!("        {:?} \n", t);
            }
            print!("        parents: {:?} \n\n", c.parents);
        }
        print!("dirty_unions {:?}\n", self.dirty_unions);
    }

    ///return the eclass of a given id
    pub fn get_eclass(&self, id: &Id) -> Option<&EClass>{
        return self.classes.get(id);
    }
    ///return a copy of the eclass of a given id
    pub fn get_eclass_cpy(&self, id: Id) -> Option<EClass>{
        if let Some(c) = self.classes.get(&id){
            return Some(c.clone());
        }
        return None;
    }

}

//basic egraph manipulation
impl EGraph{
    ///returns the canonical id and updates it for the given id
    pub fn find_mut(&mut self, id:Id)-> Id{
        return self.unionfind.find_mut(id);
    }

    ///returns the canonical id
    pub fn find(&self, id:Id)-> Id{
        return self.unionfind.find(id);
    }

    /// Checks if 2 given terms are in the same class.
    /// Panics if either of the terms aren't present in the entire egraph.
    pub fn in_same_class(&self, t1: &Enode, t2: &Enode) -> bool{
        if let Some(id1) = self.memo.get(t1){
            if let Some(id2) = self.memo.get(t2){
                return self.unionfind.in_same_set(*id1, *id2);
            }
        }
        return false
    }

    /// canonicalizes the args of a given term
    pub fn canonicalize_args(&self, term: &Enode) -> Enode{
        let mut new = term.clone();
        new.args = Vec::<Id>::new();
        for i in &term.args{
            new.args.push(self.unionfind.find(i.clone()));
        }
        return new;
    }

    ///canonicalizes an enode, updates its memo entry, and returns it
    ///does not update the eclasses
    fn canonicalize_in_memo(&mut self, n: &Enode) -> Enode{
        if let Some(old_cid) = self.memo.swap_remove(n){
            let new_cid = self.find_mut(old_cid);
            let newn = self.canonicalize_args(n);
            self.memo.insert(newn.clone(), new_cid);
            return newn;
        } else{
            return self.canonicalize_args(n);
        }
    }

    ///finds the corresponding Eclass Id of a given enode
    ///returns None if it cant find the Eclass.
    pub fn lookup(&self, enode: &Enode) -> Option<Id>{
        if let Some(id) = self.memo.get(enode){
            return Some(self.find(*id));
        }
        let enode = self.canonicalize_args(enode);
        if let Some(id) = self.memo.get(&enode){
            return Some(self.find(*id));
        }
        //if class isnt found in memo, proceed the task of
        //checking in every eclass
        for (cid, cls) in &self.classes{
            for n in &cls.nodes{
                let n = self.canonicalize_args(n);
                if n == enode{
                    return Some(*cid);
                }
            }
        }
        return None;
    }


    /// Push a potentially new eclass to the graph, then return Id
    pub fn push_eclass(&mut self, enode: &mut Enode) -> Id{
        let id = self.lookup(enode);
        if let Some(id) = id{ //term already in the graph
            return self.find_mut(id);
        }
        let id = self.unionfind.new_set();
        let eclass = EClass::new(enode.clone());


        self.memo.insert(enode.clone(), id);

        for &child in &enode.args{ // set parent pointers
            let idx = self.classes.get_index_of(&child).unwrap();
            self.classes[idx].parents.push((enode.clone(), id));
        }
        self.classes.insert(id, eclass);
        return id;
    }

    ///unions 2 eclasses and returns the new canonical Id
    ///returns None if the 2 classes are already in the same class
    pub fn union(&mut self, id1: Id, id2: Id) -> Option<Id> {
        let (id1, id2) = (self.find_mut(id1), self.find_mut(id2));
        if id1 == id2{return None }

        let id3 = self.unionfind.union(id1, id2);
        self.dirty_unions.push(id3); // id3 will need it's parents processed in rebuild!


        let (to_id, from_id) = (id1, id2);
        let from = self.classes.get(&id2).unwrap().clone();
        let mut to = self.classes.get(&id1).unwrap().clone();

        // move nodes and parents from eclass[`from`] to eclass[`to`]
        to.nodes.extend(from.nodes);
        to.parents.extend(from.parents);


        let nodes_temp = to.nodes.clone();
        to.nodes = IndexSet::<Enode>::new();
        // recanonize all nodes in memo.
        for t in &mut nodes_temp.into_iter(){
            let n = self.canonicalize_in_memo(&t);
            to.nodes.insert(n.clone());
        }

        self.classes.insert(to_id, to); //replace old `to` with new `to`
        self.classes.swap_remove(&from_id); //remove old `from`
        return Some(id3);
    }


    ///repairs the eclass of a given id
    fn repair(&mut self, id:Id){
        let id = self.find_mut(id);
        let old_cls = self.classes.get(&id).unwrap();
        let old_parents = old_cls.parents.clone();
        let old_nodes = old_cls.nodes.clone();
        let mut new_cls = EClass::empty();

        //canonicalize the eclass
        for n in old_nodes{
            new_cls.nodes.insert(self.canonicalize_args(&n));
        }
        let mut tofix = Vec::<(Enode, Id)>::new();
        for n in new_cls.nodes.clone(){
            //add own enodes to the tofix list, because this class can
            // have become congruent with parents
            tofix.push((n, id));
        }

        //  for every parent, update the hash cons. We need to repair that the term has possibly a wrong id in it
        for (t, old_cid) in old_parents{
            let newn = self.canonicalize_in_memo(&t);
            let new_cid = self.find_mut(old_cid);
            tofix.push((newn, new_cid));
        }

        // now we need to discover possible new congruence equalities in the parent nodes.
        // we do this by building up a parent hash to see if any new terms are generated.
        let mut new_parents = IndexMap::<Enode, Id>::default();
        for (newn,new_cid) in tofix {

            if let Some(n_id) = new_parents.get(&newn) {
                self.union(new_cid, *n_id);
            }
            new_parents.insert(newn.clone(), self.find_mut(new_cid));
        }
        for n in new_parents{
            new_cls.parents.push(n);
        }
        self.classes.insert(id, new_cls);
    }

    //calls repair on "duty unions"
    //which first canonicalizes some nodes in memo
    //and then checks for additional congruences that might have formed
    pub fn rebuild(&mut self){
        while self.dirty_unions.len() > 0{
            let mut todo = IndexSet::<Id>::new();
            for i in self.dirty_unions.clone(){
                todo.insert(self.find_mut(i));
            }
            for i in 0..self.unionfind.len(){
                todo.insert(self.find_mut(itoid!(i)));
            }
            self.dirty_unions = Vec::<Id>::new();
            for id in &todo{
                self.repair(*id)
            }
            // self.unionfind.canonicalize();
        }
    }


    // Push a potentially new leaflet eclass to the graph, then return Id
    pub fn leaflet(&mut self, x: String) -> Id{
        let mut t = Enode::new(x);
        return self.push_eclass(&mut t);
    }




    //insert a new Sexpr into the Egraph and return the id of the root eclass
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
            return self.leaflet(format!("{}", s));
        }
    }
}



impl EGraph{
    ///inefficient but relatively simple pattern matching algorithm
    ///returns a "dict" that translates patterns to enodes
    ///if return None,       no match was found
    ///if return Some{ {} }, one or more matches were found but the provided pattern has no variables
    ///if return Some{d},    one or more matches were found
    pub(crate) fn match_pattern(&self, cls: &EClass, p: &Pattern, sub: &mut IndexMap<Enode, Id>) -> Option<IndexMap<Pattern, Enode>> {
        if let Pattern::PatVar(s) = p {
            let mut m = IndexMap::<Pattern, Enode>::default();
            for t in &cls.nodes {
                m.insert(Pattern::PatVar(s.clone()), t.clone()); //probably 1
                if let Some(id) = self.lookup(&mut t.clone()) {
                    sub.insert(t.clone(), id);
                } else {panic!("failed to find eclass {:?}\n", t)}
            }
            return Some(m);
        }
        else if let Pattern::PatTerm(p_head, p_args) = p {
            //returns the first found matching term
            for t in &cls.nodes{ //for each node in the eclass
                if t.head != *p_head || t.len() != p_args.len(){ //no match
                    continue;
                }
                else { //heads the same, check kids
                    let mut subres = Vec::<IndexMap<Pattern, Enode>>::new();
                    let mut matching_children = 0;

                    for (t1, p1) in t.args.iter().zip(p_args){ //for each arg in the term

                        let new = self.find(*t1);
                        let c = self.get_eclass_cpy(new).unwrap(); //derive eclass from t.arg
                        let d = self.match_pattern(&c,p1, sub);
                        if let Some(d) = d {
                            subres.push(d);
                            matching_children +=1;
                        }
                    }
                    if matching_children == t.len(){
                        return EGraph::merge_consistent( subres );
                    }
                }
            }
        }
        return None;
    }

    /// same as match_pattern but only matches 1 single enode.
    /// this is a workaround to a niche case where 2 enodes in a eclass match, only the first enode match is returned.
    /// this results in a possibility where not all matches are returned.
    /// when this happens the optimal rewrite can fail to be inserted into the egraph.
    pub(crate) fn match_pattern_with_enode(&self, n: &Enode, p: &Pattern, sub: &mut IndexMap<Enode, Id>) -> Option<IndexMap<Pattern, Enode>> {
        if let Pattern::PatVar(s) = p {
        let mut m = IndexMap::<Pattern, Enode>::default();
            m.insert(Pattern::PatVar(s.clone()), n.clone()); //probably 1
            if let Some(id) = self.lookup(&mut n.clone()) {
                sub.insert(n.clone(), id);
            } else {panic!("failed to find eclass {:?}\n", n)}
        return Some(m);
        }
        else if let Pattern::PatTerm(p_head, p_args) = p {
            //returns the first found matching term
            if n.head != *p_head || n.len() != p_args.len(){ //no match
                return None;
            }
            else { //heads the same, check kids
                let mut subres = Vec::<IndexMap<Pattern, Enode>>::new();
                let mut matching_children = 0;
                for (t1, p1) in n.args.iter().zip(p_args){ //for each arg in the term
                    let new = self.find(*t1);
                    let c = self.get_eclass_cpy(new).unwrap(); //derive eclass from t.arg
                    let d = self.match_pattern(&c,p1, sub);
                    if let Some(d) = d {
                        subres.push(d);
                        matching_children +=1;
                    }
                }
                if matching_children == n.len(){
                    return EGraph::merge_consistent( subres );
                }
            }
        }
        return None;
    }

    ///merge dicts and return None if a inconsistency is found
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

    ///sub maps a VAR symbol to an Identifier in the EGraph
    ///this means that the VAR symbol comes from pattern and the ID from the EGraph
    fn instantiate(&mut self, p: Pattern, translator: &IndexMap<Pattern, Enode>, sub: &IndexMap<Enode, Id>) -> Option<Id>{
        if let Pattern::PatVar(s) = p { //adding a patvar to the EG

            if let Some(t) = translator.get(&Pattern::PatVar(s.clone())){
                let tid = sub.get(t); //get the appropriate ID
                return Some(*tid.unwrap()); //return the appropriate ID
            }
            else {
                return None;
            }
        } else if let Pattern::PatTerm(p_head, p_args) = p {
            let mut t = Enode::new(p_head.clone()); //get term head
            for a in p_args { //get terms in p_args
                
                //recursive call, return val is the ID of the Enode the call added to the EGraph
                if let Some(id) = self.instantiate(*a, translator, &sub){
                    t.args.push(id);//push that ID to our term
                }
                else{
                    return None
                }
            }
            return Some(self.push_eclass(&mut t)); //push term into EGraph
        }
        return None; //unreachable
    }

    ///matches the given rule.lhs once with each eclass
    ///inserts rule.rhs whereever a match is found
    ///returns the number of times a rule is succesfully applied to an eclass
    pub fn rewrite_lhs_to_rhs(&mut self, r: &Rule) -> i32{
        let mut bufdict = IndexMap::<Enode, Id>::default();
        //let mut translator = IndexMap::<Pattern, Enode>::default();
        let lhs = r.lhs.clone();
        let rhs = r.rhs.clone();

        let mut edits = 0; //used for statistics
        let mut matchvec = Vec::<Option<IndexMap<Pattern, Enode>>>::new();
        for (_, cls) in &self.classes{
            for n in &cls.nodes{
                matchvec.push(self.match_pattern_with_enode(&n, &lhs, &mut bufdict));
            }
        }
        for matches in matchvec {
            if matches.is_some() {
                let translator = matches.unwrap();
                if let Some(id1) =  self.instantiate(lhs.clone(), &translator,&bufdict){
                    if let Some(id2) =  self.instantiate(rhs.clone(), &translator,&bufdict){
                        if let Some(_) = self.union(id1, id2){
                            edits += 1;
                        }
                    }
                }
            }
        }
        return edits;
    }

    ///applies all rules in a passed ruleset to the egraph.
    ///rebuilds after every rule applucation.
    ///note that edits dont directly correlate to how much an egraph is mutated.
    ///this is because rebuild doesn't keep track of its edits
    pub fn rewrite_ruleset(&mut self, rs:&Vec<Rule>)-> i32{
        let mut edits = 0;
        for r in rs{
            edits += self.rewrite_lhs_to_rhs(r);
            self.rebuild();
        }
        return edits;
    }
}

/* --------------------- */
//the extraction algorithm
/* --------------------- */
impl EGraph{
    
    ///depth first extract with a visited list to prevent infinitely looping
    pub fn extract(&self, visited: &Vec<Id>, cid: Id, cost_function: &dyn Fn(&String) -> i32) -> Option<(i32, String)> {
        let cid = self.find(cid);
        let class = self.get_eclass(&cid).unwrap();
        let mut evaluation = Vec::<(i32, String)>::new();
        
        //if already visited, skip, otherwise add own id and continue
        if visited.contains(&cid){
            return None;
        }

        let mut visited = visited.clone();
        visited.push(cid);

        for n in class.nodes.clone(){
            let mut cost = cost_function(&n.head);
            if n.len() == 0 {
                evaluation.push((cost, n.head));
                continue;
            }
            let mut str = format!("({}", &n.head);
            let mut eval = true;
            for arg in n.args{
                if let Some((res, s)) = self.extract(&visited, arg, cost_function){
                    cost += res;
                    str.push_str(&format!(" {}", s));
                } else{
                    eval = false;
                    break;
                }
            }
            if eval{
                evaluation.push((cost, format!("{})", str)));
            }
        }
        //after evaluating each enode extract the cheapest one from the list
        if let Some(tn) = evaluation.iter().min_by_key(|d|d.0){
            return Some(tn.clone());
        }
        return None
        
    }

    /// an example of an extraction function.
    /// root_id is the id of the eclass you want to start the extraction from
    pub fn extract_shortest(&self, root_id: Id) -> Option<String>{
        let cost_function= &ret_1;
        return self.extract_best(root_id, cost_function)
    }

    /// an example of an extraction function.
    /// root_id is the id of the eclass you want to start the extraction from
    pub fn extract_logical(&self, root_id: Id) -> Option<String>{
        let cost_function= &ret_logical;
        return self.extract_best(root_id, cost_function)
    }

    /// extract and retur the best sexpr from the egraph according to cost_function
    /// returns the answer as a string
    pub fn extract_best(&self, root_id: Id, cost_function: &dyn Fn(&String) -> i32) -> Option<String>{
        let canonical_root_id = self.find(root_id);
        let visited = Vec::<Id>::default();
        if let Some((_, res)) = self.extract(&visited, canonical_root_id, cost_function){
            return Some(res);
        }
        return None;
    }

}

/* --------------------- */
//some cost function examples

/// example cost_function
/// always returns a cost of 1
pub fn ret_1(_: &String) -> i32{
    return 1;
}

/** example cost function
 returns a cost based on the value of the operation:
 "+, n, <<" => 2,
 "/" => 4,
 "*" => 3,
 "succ" => 0,
 _ => 1 **/
pub fn ret_logical(s: &String) -> i32{
    return match s.as_str(){
        "+" => 2,
        "/" => 4,
        "*" => 3,
        "<<" => 2,
        "succ" => 0,
        "n" => 2,
        _ => 1 //unknown ops and loading consts/variables
    };
}
