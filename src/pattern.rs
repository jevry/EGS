
//used for pattern matching

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Pattern {
    PatVar(String),
    PatTerm(String, Vec<Box<Pattern>>),
}

pub struct Rule {
    pub lhs: Pattern,
    pub rhs: Pattern
}