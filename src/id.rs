/*
 * eclass.rs
 * -------------------------
 * Author  : Kieran van Gelder
 * Id      : 14033623
 * Date    : 2024
 * Version : 0.1
 * -------------------------
 * manually define an ID for easier oversight of the code
 * we also use this struct in other
 */

#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Id(usize);
impl From<usize> for Id { //cast usize to Id, used as Id::from(n)
    fn from(n: usize) -> Id {
        return Id(n as usize);
    }
}
impl From<Id> for usize { //cast Id to usize, used as usize::from(n)
    fn from(id: Id) -> usize {
        return id.0 as usize;
    }
}

//
#[macro_export]//export macro
macro_rules! itoid { // Int_TO_ID; simplify Id::from function
    ( $($x:expr)? ) =>{
        $( Id::from($x) )+
    };
}
