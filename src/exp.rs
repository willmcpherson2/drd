#[derive(Debug, PartialEq, Clone)]
pub enum Exp {
    Let(String, Box<Exp>, Box<Exp>),
    Select(Vec<String>, Box<Exp>),
    Where(Box<Exp>, Box<Exp>),
    Union(Box<Exp>, Box<Exp>),
    Difference(Box<Exp>, Box<Exp>),
    Product(Box<Exp>, Box<Exp>),
    Table(Vec<String>, Vec<Exp>),
    Or(Box<Exp>, Box<Exp>),
    Equals(Box<Exp>, Box<Exp>),
    And(Box<Exp>, Box<Exp>),
    Not(Box<Exp>),
    Bool(bool),
    Int(i64),
    Str(String),
    Var(String),
}
