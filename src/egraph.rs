




use symbolic_expressions::Sexp;
use crate::Id;
use crate::Enode;
use crate::UnionFind;
use crate::EClass;
use crate::mstr;
use crate::pattern::{Pattern, Rule};
use indexmap::IndexSet;

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
    pub fn get_eclass(&self, id: Id) -> Option<&EClass>{
        return self.classes.get(&id);
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

    // canonicalizes the args of a given term, then returns it
    pub fn canonicalize_args(&mut self, term: &mut Enode) -> Enode{
        let mut new = Vec::<Id>::new();
        for i in &term.args{
            new.push(self.unionfind.find_mut(i.clone()));
        }
        term.args = new;
        return term.clone();
    }

    //finds the corresponding Eclass Id of a given enode
    //returns None if it cant find the Eclass.
    //NOTE: Currently overcomplicated because canonicalize_args
    //doesnt properly update the egraph, as a result there is no garuantee wheter
    //the node is stored canonicalized or not
    pub fn find_eclass(&mut self, enode: &mut Enode) -> Option<Id>{
        if let Some(id) = self.memo.get(enode){
            return Some(self.find_mut(*id));
        }
        let t = self.canonicalize_args(enode);
        if let Some(id) = self.memo.get(&t){
            return Some(self.find_mut(*id));
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
            self.classes[idx].parents.push(enode.clone());
        }
        self.classes.insert(id, eclass);
        return id;
    }

    //unions 2 eclasses and returns the new canonical Id
    //returns None if the 2 classes are already in the same class
    pub fn union(&mut self, id1: Id, id2: Id) -> Option<Id> {
        let (id1, id2) = (self.find_mut(id1), self.find_mut(id2));
        if id1 == id2{ return None }
        
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
            let &tid = self.memo.get(t).unwrap();
            self.memo.swap_remove(t);
            let t = self.canonicalize_args(t);
            self.memo.insert(t.clone(), tid);
            to.nodes.push(t);
        }

        self.classes.insert(to_id, to); //replace old `to` with new `to`
        self.classes.swap_remove(&from_id); //remove old `from`
        return Some(id3);
    }


    fn repair(&mut self, id:Id){
        //clone the entire eclass, extremely inefficient but rust refuses to play nice otherwise
        let mut new_cls = self.classes.get(&id).unwrap().clone();
        self.classes.swap_remove(&id);
        let parents = new_cls.parents.clone();
        new_cls.parents = Vec::<Enode>::new(); //clear parents

        //  for every parent, update the hash cons. We need to repair that the term has possibly a wrong id in it
        for mut t in parents{
            if let Some(old_parent_id) = self.memo.swap_remove(&t){ // the parent should be updated to use the updated class id
                new_cls.parents.push(self.canonicalize_args(&mut t)); //canonicalize
                self.memo.insert(t.clone(), old_parent_id); // replace in hash cons
            }
        }
        self.classes.insert(id, new_cls);

        // now we need to discover possible new congruence equalities in the parent nodes.
        // we do this by building up a parent hash to see if any new terms are generated.
        // new_parents = Dict()
        // for (t,t_id) in cls.parents {
        //     self.canonicalize_args(t) // canonicalize. Unnecessary by the above?
        //     if haskey(new_parents, t){
        //         self.union(t_id, new_parents[t])
        //     }

        //     new_parents[t] = self.find_root!(e, t_id)
        // }

        // e.classes[id].parents = [ (p,id) for (p,id) in new_parents]
    }

    //calls repair on "duty unions"
    //which first canonicalizes some nodes in memo
    //(NOT IMPLEMENTED:) and then checks for additional congruences that might have formed
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



//inefficient but relatively simple pattern matching algorithm
//returns a "dict" that translates patterns to enodes
//if return None,       no match was found
//if return Some{ {} }, one or more matches were found but the provided pattern has no variables
//if return Some{d},    one or more matches were found
impl EGraph{
    fn match_pattern(&mut self, e: &EClass, p: &Pattern, sub: &mut IndexMap<Enode, Id>) -> Option<IndexMap<Pattern, Enode>> {
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
                        let new = self.find_mut(*t1);
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
            let t = translator.get(&Pattern::PatVar(s));
            let tid = sub.get(t.unwrap()); //get the appropriate ID
            return Some(*tid.unwrap()); //return the appropriate ID

        } else if let Pattern::PatTerm(p_head, p_args) = p {
            let mut t = Enode::new(p_head); //get term head
            for a in p_args { //get terms in p_args

                //recursive call, return val is the ID of the Enode the call added to the EGraph
                t.args.push(self.instantiate(*a, translator, &sub).unwrap());//push that ID to our term
            }
            return Some(self.push_eclass(&mut t)); //push term into EGraph
        }
        return None; //instantiate breaks
    }

    //matches the given rule.lhs once with each eclass
    //inserts rule.rhs whereever a match is found
    //returns the number of times a rule is succesfully applied to an eclass
    pub fn rewrite_lhs_to_rhs(&mut self, r: &Rule) -> i32{
        let mut bufdict = IndexMap::<Enode, Id>::default();
        //let mut translator = IndexMap::<Pattern, Enode>::default();
        let lhs = r.lhs.clone();
        let rhs = r.rhs.clone();

        let mut edits = 0;
        for (_, cls) in self.classes.clone(){
            let matches =  self.match_pattern(&cls, &lhs, &mut bufdict);
            if matches.is_some() {
                edits += 1;
                let temp = matches.unwrap();
                let id1 =  self.instantiate(lhs.clone(), &temp,&bufdict);
                let id2 =  self.instantiate(rhs.clone(), &temp,&bufdict);
                self.union(id1.unwrap(), id2.unwrap());

            }
        }
        return edits;
    }

    //applies all rules in a passed ruleset to the egraph
    pub fn rewrite_ruleset(&mut self, rs:&Vec<Rule>)-> i32{
        let mut edits = 0;
        for r in rs{
            edits += self.rewrite_lhs_to_rhs(r);
        }
        
        return edits;
    }


    fn extract_shortest(&self, root_id: Id) -> Option<String>{
        let c_root_id = self.find(root_id);
        let cost_function= &ret_1;
        if let Some(c) = self.get_eclass_cpy(c_root_id){
            let (_, res) = self.extract(c, cost_function);
            return Some(res);
        }
        return None;
    }

    pub fn extract(&self, c: EClass, cost_function: &dyn Fn(&String) -> i32) -> (i32, String) {
        let mut todo = Vec::<Enode>::new();
        for n in c.nodes.clone(){
            if n.len() == 0{
                return (cost_function(&n.head), n.head);
            } else {
                todo.push(n);
            }
        }

        let mut evaluation = Vec::<(i32, String)>::new();
        for n in todo{
            let mut cost = cost_function(&n.head);
            let mut str = format!("({}", &n.head);
            for arg in n.args{
                let c_root_id = self.find(arg);
                if let Some(c) = self.get_eclass_cpy(c_root_id){
                    let (res, s) = self.extract(c, cost_function);
                    cost += res;
                    str.push_str(&format!(" {}", s));
                }
            }
            evaluation.push((cost, format!("{})", str)));
        }
        let tn = evaluation.iter().min_by_key(|d|d.0).unwrap();
        
        return tn.clone();
    }
}


/* --------------------- */
//some cost function examples
//it is reccomended to keep minimum cost at 1 to encourage short terms

//always returns a cost of 1
fn ret_1(s: &String) -> i32{
    return 1;
}

//returns a cost based on the value of the operation
fn ret_logical(s: &String) -> i32{
    return match s.as_str(){
        "/" => 4,
        "*" => 3,
        "<<" => 2,
        _ => 1 //unknown ops and loading consts/variables
    };
}
/* --------------------- */


//run these tests on your local machine
#[cfg(test)]
mod tests {
    use symbolic_expressions::parser::parse_str;
    use symbolic_expressions::parser;
    use crate::pattern::read_ruleset;
    use crate::pattern::new_pattern;
    use crate::itoid;

    use super::*; //allows this module to use previous scope
    static PATH: &str = "src/testsuite/";
    static FILENAME: &str = "ints/example.txt";

    #[test] //run this test function to see graph construction
    fn egraph_construction() {
        let filepath = format!("{PATH}{FILENAME}");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        print!("empty graph: {:?}\n", g);
        let root = g.insert_sexpr(sexp);

        print!("\nnew graph: ");
        g.print();
        print!("root: {:?}\n", root);
    }



    //run this test function to see adding a new term to a constructed graph
    //note that doing this like this breaks the congruence invariant
    #[test]
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

    #[test] //run this test function to see adding unioning 2 nodes
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


    #[test] //to test ematching
    fn egraph_matching() {
        let filepath = format!("{PATH}ints/const_mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();

        let sexp1 = parse_str("(* P_a 2)").unwrap();
        let patt = new_pattern(sexp1.clone()).unwrap();

        let mut dict = IndexMap::<Enode, Id>::default();

        for (_, c) in g.classes.clone(){
            print!("\nres = {:?}\n", g.match_pattern(&c, &patt, &mut dict));
        }
        print!("var map = {:?}\n\n", dict);

    }

    #[test] //to test rewriting a graph once
    fn egraph_rewrite() {
        let filepath = format!("{PATH}ints/mult.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();

        let sexp1 = parse_str("(* P_x 2)").unwrap();
        let sexp2 = parse_str("(<< P_x 1)").unwrap();
        let r = &Rule::new_rule(sexp1, sexp2).unwrap();
        g.rewrite_lhs_to_rhs(r);
        g.print();
    }

    #[test] //to test rewriting a graph multiple times
    pub fn egraph_mass_rewrite() {
        let filepath = format!("{PATH}ints/example.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        g.insert_sexpr(sexp);
        print!("\nnew egraph: ");
        g.print();


        let ruleset = &read_ruleset(format!("src/rulesets/rulesetA.txt"));
        let edits = g.rewrite_ruleset(ruleset);

        print!("first pass edits: {}\n", edits);
        let edits = g.rewrite_ruleset(ruleset);

        print!("second pass edits: {}\n", edits);
        let edits = g.rewrite_ruleset(ruleset);
        print!("third pass edits made: {}\n", edits);
        g.rebuild();
        g.print();
    }



    #[test] //rewrite a term a few times and extract some random terms from the graph
    pub fn extract_random_terms(){
        let filepath = format!("{PATH}ints/example.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        let root_id = g.insert_sexpr(sexp);
        let ruleset = &read_ruleset(format!("src/rulesets/rulesetA.txt"));
        g.rewrite_ruleset(ruleset);
        g.rebuild();
        g.rewrite_ruleset(ruleset);
        g.rebuild();
        g.rewrite_ruleset(ruleset);
        g.rebuild();
        g.print();
        
        if let Some(cls) = g.get_eclass_cpy(root_id){
            for n in cls.nodes{
                print!("res: {}\n", extract_random(&g, n));
            }
        }
    }

    use rand::{thread_rng, Rng};
    fn extract_random(e: &EGraph, n: Enode) -> String{
        let mut rng = thread_rng();
        let mut ans = String::new();
        if n.len() == 0 {return format!(" {}", n.head);}
        ans.push_str(&format!(" ( {} ", n.head));

        for id in n.args{
            let cid = e.find(id);
            if let Some(c) = e.get_eclass_cpy(cid){
                
                let ran = rng.gen_range(0..c.nodes.len());
                let n2 = c.nodes[ran].clone();
                ans.push_str(&extract_random(e, n2));
            }
        }
        ans.push_str(&format!(")"));
        return ans;
    }

    #[test] //extracts the best found term from a set of options
    fn term_extraction(){
        let filepath = format!("{PATH}ints/example.txt");
        let sexp: Sexp = parser::parse_file(&filepath).unwrap();
        let mut g = EGraph::new();
        let root_id = g.insert_sexpr(sexp);
        let ruleset = &read_ruleset(format!("src/rulesets/rulesetA.txt"));
        g.rewrite_ruleset(ruleset);
        // g.rebuild();
        g.rewrite_ruleset(ruleset);
        // g.rebuild();
        g.rewrite_ruleset(ruleset);
        // g.rebuild();
        g.rewrite_ruleset(ruleset);
        g.rebuild();
        g.print();
        

        print!("res: {:?}\n", g.extract_shortest(root_id));
    }
}
