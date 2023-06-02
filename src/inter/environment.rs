use anyhow::{bail, Result};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::inter::{ast::Expression, evaluator::EvaluationError};

use super::object::Object;

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_with_outer(outer: Rc<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn set_arguments_to_env(
        &mut self,
        args: Vec<Object>,
        params: Vec<Expression>,
    ) -> Result<()> {
        for (idx, arg) in args.into_iter().enumerate() {
            let name = self.get_name(&params[idx])?;
            self.store.insert(name, arg);
        }

        Ok(())
    }

    fn get_name(&self, name: &Expression) -> Result<String> {
        let name = match name {
            Expression::Identifier(s) => s,
            _ => bail!(EvaluationError::ExpectedIdentifier {
                found: name.type_as_string()
            }),
        };
        Ok(name.to_string())
    }

    pub fn get(&self, name: &str) -> Result<Object> {
        let res = match self.store.get(name) {
            Some(o) => o,
            None => match self.outer {
                Some(ref outer_env) => {
                    let outer_env = outer_env.borrow();
                    return outer_env.get(name);
                }
                None => bail!(EvaluationError::IdentifierNotFound {
                    identifier: name.to_string()
                }),
            },
        };
        Ok(res.clone())
    }

    pub fn set(&mut self, name: &Expression, value: Object) -> Result<()> {
        let name = self.get_name(name)?;
        self.store.insert(name, value);
        Ok(())
    }
}
