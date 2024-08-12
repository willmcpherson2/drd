#[derive(Debug, PartialEq, Clone)]
pub struct Program(pub Vec<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub enum Exp {
    Let(Let),
    Select(Select),
    Where(Where),
    Union(Union),
    Difference(Difference),
    Product(Product),
    Table(Table),
    Row(Row),
    Cell(Cell),
    Or(Or),
    Equals(Equals),
    And(And),
    Not(Not),
    Bool(Bool),
    Int(Int),
    Str(Str),
    Var(Var),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Let(pub Var, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Select(pub Vec<Var>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Where(pub Box<Exp>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Union(pub Box<Exp>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Difference(pub Box<Exp>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Product(pub Box<Exp>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Table(pub Box<Exp>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Row(pub Box<Exp>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Cell(pub Var, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Or(pub Box<Exp>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Equals(pub Box<Exp>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct And(pub Box<Exp>, pub Box<Exp>);

#[derive(Debug, PartialEq, Clone)]
pub struct Not(pub Box<Exp>);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Bool(pub bool);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Int(pub i64);

#[derive(Debug, PartialEq, Clone)]
pub struct Str(pub String);

#[derive(Debug, PartialEq, Clone)]
pub struct Var(pub String);
