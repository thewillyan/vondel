use std::{cell::RefCell, fmt, rc::Rc};

use enum_as_inner::EnumAsInner;

use crate::inter::{
    ast::{Expression, StatementType},
    environment::Environment,
};

#[derive(Debug, PartialEq, EnumAsInner, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    ReturnValue(Box<Object>),
    Null,
    Function {
        params: Vec<Expression>,
        body: StatementType,
        env: Rc<RefCell<Environment>>,
    },
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.to_string(),
            Object::Boolean(b) => b.to_string(),
            Object::Null => String::from("null"),
            Object::ReturnValue(v) => v.inspect(),
            Object::Function { .. } => "".to_string(),
        }
    }

    pub fn type_as_string(&self) -> &'static str {
        match self {
            Object::Integer(_) => "Integer",
            Object::Boolean(_) => "Boolean",
            Object::Null => "Null",
            Object::ReturnValue(_) => "ReturnValue",
            Object::Function { .. } => "Function",
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf = match self {
            Object::Integer(v) => v.to_string(),
            Object::Boolean(v) => v.to_string(),
            Object::Null => String::from("null"),
            Object::ReturnValue(v) => v.to_string(),
            Object::Function { .. } => String::from("fn"),
        };
        write!(f, "{}", buf)
    }
}
