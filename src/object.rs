use crate::call::Func;

use std::fmt::{
    Debug,
    Formatter
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Function,
    Int,
    Bool,
    Str,
    List,
    None,
    Any
}

impl Debug for Type {
    fn fmt(&self, form: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(form, "{}", match self {
            Type::Any => "any",
            Type::Function => "function",
            Type::Int => "int",
            Type::Bool => "bool",
            Type::Str => "string",
            Type::List => "list",
            Type::None => "none_t"
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Object {
    Function(Func),
    Int(i64),
    Bool(bool),
    Str(String),
    List(Vec<Object>),
    None
}
