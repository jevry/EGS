




use std::default;
use std::vec;

use symbolic_expressions::Sexp;
use crate::Id;
use crate::Enode;
use crate::UnionFind;
use crate::EClass;
use crate::mstr;
use crate::pattern::{Pattern, Rule};
use indexmap::IndexSet;
use bimap::BiMap;

//we use hasmap instead of indexmap because indexmaps are more deterministic and easier to debug
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

    //return the eclass of a given id
    pub fn get_eclass(&self, id: &Id) -> Option<&EClass>{
        return self.classes.get(id);
    }
    //return a copy of the eclass of a given id
    pub fn get_eclass_cpy(&self, id: Id) -> Option<EClass>{
        if let Some(c) = self.classes.get(&id){
            return Some(c.clone());
        }
        return None;
    }

}

//basic egraph manipulation
impl EGraph{
    //returns the canonical id and updates it for the given id
    pub fn find_mut(&mut self, id:Id)-> Id{
        return self.unionfind.find_mut(id);
    }

    //returns the canonical id
    pub fn find(&self, id:Id)-> Id{
        return self.unionfind.find(id);
    }

    // Checks if 2 given terms are in the same class.
    // Panics if either of the terms aren't present in the entire egraph.
    pub fn in_same_class(&self, t1: &Enode, t2: &Enode) -> bool{
        let (_, id1) = self.memo.get_key_value(t1).unwrap();
        let (_, id2) = self.memo.get_key_value(t2).unwrap();
        return self.unionfind.in_same_set(*id1, *id2);
    }

    // canonicalizes the args of a given term
    pub fn canonicalize_args(&self, term: &mut Enode) -> Enode{
        let mut new = Vec::<Id>::new();
        for i in &term.args{
            new.push(self.unionfind.find(i.clone()));
        }
        term.args = new;
        return term.clone();
    }

    //finds the corresponding Eclass Id of a given enode
    //returns None if it cant find the Eclass.
    //NOTE: Currently overcomplicated because canonicalize_args
    //doesnt properly update the egraph, as a result there is no garuantee wheter
    //the node is stored canonicalized or not
    pub fn find_eclass(&self, enode: &mut Enode) -> Option<Id>{
        if let Some(id) = self.memo.get(enode){
            return Some(self.find(*id));
        }
        self.canonicalize_args(enode);
        if let Some(id) = self.memo.get(enode){
            return Some(self.find(*id));
        }
        return None;
    }

    // Push a potentially new eclass to the graph, then return Id
    pub fn push_eclass(&mut self, enode: &mut Enode) -> Id{
        let id = self.find_eclass(enode);
        if id.is_some(){ //term already in the graph
            return id.unwrap();
        }
        let id = self.unionfind.new_set();
        let eclass = EClass::new(enode.clone());


        self.memo.insert(enode.clone(), id);

        for &child in &enode.args{ // set parent pointers
            let idx = self.classes.get_index_of(&child).unwrap();
            self.classes[idx].parents.insert(enode.clone(), id);
        }
        self.classes.insert(id, eclass);
        return id;
    }

    //unions 2 eclasses and returns the new canonical Id
    //returns None if the 2 classes are already in the same class
    pub fn union(&mut self, id1: Id, id2: Id) -> Option<Id> {
        let (id1, id2) = (self.find_mut(id1), self.find_mut(id2));
        if id1 == id2{ print!("!!!failed union!!!\n\n"); return None }
        print!("---succesfull union---\n\n");

        let id3 = self.unionfind.union(id1, id2);
        self.dirty_unions.push(id3); // id3 will need it's parents processed in rebuild!


        let (to_id, from_id) = (id1, id2);
        let from = self.classes.get(&id2).unwrap().clone();
        let mut to = self.classes.get(&id1).unwrap().clone();

        // move nodes and parents from eclass[`from`] to eclass[`to`]
        to.nodes.extend(from.nodes);
        to.parents.extend(from.parents);


        let mut temp = to.nodes.clone();
        to.nodes = Vec::<Enode>::new();
        // recanonize all nodes in memo.
        for t in &mut temp{
            if let Some(mut tid) = self.memo.get(t){
                let mut tid = *tid;
                let tid = self.find_mut(tid);
                self.memo.swap_remove(t);
                self.canonicalize_args(t);
                self.memo.insert(t.clone(), tid);
                to.nodes.push(t.clone());
            }
            
        }

        self.classes.insert(to_id, to); //replace old `to` with new `to`
        self.classes.swap_remove(&from_id); //remove old `from`
        return Some(id3);
    }


    fn repair(&mut self, id:Id){
        //clone the entire eclass, extremely inefficient but rust refuses to play nice otherwise
        let id = self.find(id);
        let mut new_cls = self.classes.get(&id).unwrap().clone();
        let parents = new_cls.parents.clone();
        
        let mut tofix = Vec::<(Enode, Id)>::new();


        print!("\nprerepair parents {:?}\n", parents);
        new_cls.parents.clear(); //clear parents
        
        //  for every parent, update the hash cons. We need to repair that the term has possibly a wrong id in it
        for (mut t, t_id) in parents{
            if let Some(old_parent_id) = self.memo.swap_remove(&t){ // the parent should be updated to use the updated class id
                print!("parent located: {:?}, {:?}\n",t , &t_id);
                let t_id = self.find_mut(t_id);
                let old_parent_id = self.find_mut(old_parent_id);
                new_cls.parents.insert(self.canonicalize_args(&mut t), t_id); //canonicalize
                tofix.push((self.canonicalize_args(&mut t), t_id));
                self.memo.insert(t.clone(), old_parent_id); // replace in hash cons
            } else { // the parent should be updated to use the updated class id
                print!("parent relocated: {:?}, {:?}\n",t , &t_id);
                tofix.push((self.canonicalize_args(&mut t), t_id));
            } 
            
        }
        print!("postrepair parents {:?}\n\n", new_cls.parents);

        // now we need to discover possible new congruence equalities in the parent nodes.
        // we do this by building up a parent hash to see if any new terms are generated.
        let mut new_parents = bimap::BiMap::<Enode, Id>::new();
        for (mut t,t_id) in tofix {
            //let t = self.canonicalize_args(&mut t); // canonicalize. Unnecessary by the above?
            let t_id = self.find(t_id);
            print!("need to union {:?}, {:?}\n", t, t_id);
            if let Some(n_id) = new_parents.get_by_left(&t) {
                print!("repairs: unioning {:?} and {:?}\n", t_id, n_id);
                self.union(t_id, *n_id);
            }
            
            new_parents.insert(t.clone(), self.find(t_id));
        }
        print!("done\n");
        new_cls.parents =  new_parents;

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
            self.dirty_unions = Vec::<Id>::new();
            for id in todo{
                self.repair(id)
            }
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
    //inefficient but relatively simple pattern matching algorithm
    //returns a "dict" that translates patterns to enodes
    //if return None,       no match was found
    //if return Some{ {} }, one or more matches were found but the provided pattern has no variables
    //if return Some{d},    one or more matches were found
    pub(crate) fn match_pattern(&self, e: &EClass, p: &Pattern, sub: &mut IndexMap<Enode, Id>) -> Option<IndexMap<Pattern, Enode>> {
        if let Pattern::PatVar(s) = p {
            let mut m = IndexMap::<Pattern, Enode>::default();
            for t in &e.nodes {
                m.insert(Pattern::PatVar(s.clone()), t.clone()); //probably 1
                if let Some(id) = self.find_eclass(&mut t.clone()) {
                    sub.insert(t.clone(), id);
                } else {panic!("failed to find eclass {:?}\n", t)}
            }
            return Some(m);
        }
        else if let Pattern::PatTerm(p_head, p_args) = p {
            for t in &e.nodes{ //for each node in the eclass
                if t.head != *p_head || t.len() != p_args.len(){ //no match
                    continue;
                }
                else { //heads the same, check kids
                    let mut res = Vec::<IndexMap<Pattern, Enode>>::new();
                    let mut matching_children = 0;

                    for (t1, p1) in t.args.iter().zip(p_args){ //for each arg in the term
                        let new = self.find(*t1);
                        let c = self.get_eclass_cpy(new).unwrap(); //derive eclass from t.arg
                        let d = self.match_pattern(&c,p1, sub);
                        if let Some(d) = d {
                            res.push(d);
                            matching_children +=1;
                        }
                    }
                    if matching_children == t.len(){
                        return EGraph::merge_consistent( res );
                    }
                }
            }
        }
        return None;
    }

    //merge dicts and return None if a inconsistency is found
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
            print!("adding {} as {:?}\n", &s, &t);
            let tid = sub.get(t.unwrap()); //get the appropriate ID

            return Some(*tid.unwrap()); //return the appropriate ID

        } else if let Pattern::PatTerm(p_head, p_args) = p {
            let mut t = Enode::new(p_head.clone()); //get term head
            for a in p_args { //get terms in p_args
                
                //recursive call, return val is the ID of the Enode the call added to the EGraph
                t.args.push(self.instantiate(*a, translator, &sub).unwrap());//push that ID to our term
            }
            print!("adding {} as {:?}\n", &p_head, &t);
            return Some(self.push_eclass(&mut t)); //push term into EGraph
        }
        return None; //unreachable
    }

    //matches the given rule.lhs once with each eclass
    //inserts rule.rhs whereever a match is found
    //returns the number of times a rule is succesfully applied to an eclass
    pub fn rewrite_lhs_to_rhs(&mut self, r: &Rule) -> i32{
        let mut bufdict = IndexMap::<Enode, Id>::default();
        //let mut translator = IndexMap::<Pattern, Enode>::default();
        let lhs = r.lhs.clone();
        let rhs = r.rhs.clone();

        let mut edits = 0; //used for statistics
        let mut matchvec = Vec::<Option<IndexMap<Pattern, Enode>>>::new();
        for (_, cls) in &self.classes{
            let matches =  self.match_pattern(&cls, &lhs, &mut bufdict);
            matchvec.push(matches);
        }
        print!("dict: {:?}\n",  &bufdict);
        for matches in matchvec {
            if matches.is_some() {
                print!("found match {:?}\n", matches);
                print!("instantiating: {:?}\n", rhs);
                edits += 1;
                let translator = matches.unwrap();
                let id1 =  self.instantiate(lhs.clone(), &translator,&bufdict);
                let id2 =  self.instantiate(rhs.clone(), &translator,&bufdict);
                self.union(id1.unwrap(), id2.unwrap());

            }
        }
        return edits;
    }

    //applies all rules in a passed ruleset to the egraph
    pub fn rewrite_ruleset(&mut self, rs:&Vec<Rule>)-> i32{
        let mut edits = 0;
        self.print();
        for r in rs{
            edits += self.rewrite_lhs_to_rhs(r);
            print!("---PRE-REPAIRS STATUS:\n\n");
            self.print();
            print!("\n---STARTING REPAIRS\n laundry: {:?}\n", self.dirty_unions);
            self.rebuild();
            print!("---FINISHING REPAIRS\n\n");
            print!("---RESULT:\n");
            self.print();
            // print!("okay\n\n");
        }
        // print!("done editing\n\n\n");
        return edits;
    }

}

/* --------------------- */
//the extraction algorithm
/* --------------------- */
impl EGraph{
    
    //depth first extract with a visited list to prevent infinitely looping
    //the visited list is just to keep track what each "line" of extract has visited.
    //an individual line can never visit the same eclass twice
    //though a splitted line might
    pub fn extract(&self, visited: &Vec<Id>, cid: Id, cost_function: &dyn Fn(&String) -> i32) -> Option<(i32, String)> {
        let cid = self.find(cid);
        let class = self.get_eclass(&cid).unwrap();
        let mut evaluation = Vec::<(i32, String)>::new();
        let mut next_visited = visited.clone();
        next_visited.push(cid);

        for n in class.nodes.clone(){
            let mut cost = cost_function(&n.head);
            let mut str = format!("({}", &n.head);
            
            if let Some(cid_of_node) = self.find_eclass(&mut n.clone()){
                if visited.contains(&cid_of_node){
                    continue;
                }
            }
            let mut eval = true;
            for arg in n.args{
                let c_root_id = self.find(arg);
                if let Some(c) = self.get_eclass_cpy(c_root_id){
                    if let Some((res, s)) = self.extract(&next_visited, arg, cost_function){
                        cost += res;
                        str.push_str(&format!(" {}", s));
                    } else{
                        eval = false;
                        continue;
                    }
                }
            }
            if eval{
                evaluation.push((cost, format!("{})", str)));
            }
        }
        
        if let Some(tn) = evaluation.iter().min_by_key(|d|d.0){
            return Some(tn.clone());
        }
        return None
        
    }

    //an example of an extraction function
    //root_id is the id of the eclass you want to start the extraction from
    pub fn extract_shortest(&self, root_id: Id) -> Option<String>{
        //canonicalize the id of the root eclass
        let canonical_root_id = self.find(root_id);
        let cost_function= &ret_1;
        let visited = Vec::<Id>::default();
        if let Some((_, res)) = self.extract(&visited, canonical_root_id, cost_function){
            return Some(res);
        }
        return None;
    }

}

/* --------------------- */
//some cost function examples
//it is reccomended to keep minimum cost at 1 to encourage short terms

//always returns a cost of 1
pub fn ret_1(_: &String) -> i32{
    return 1;
}

//returns a cost based on the value of the operation
pub fn ret_logical(s: &String) -> i32{
    return match s.as_str(){
        "+" => 2,
        "/" => 4,
        "*" => 3,
        "<<" => 2,
        "succ" => 0,
        _ => 1 //unknown ops and loading consts/variables
    };
}
