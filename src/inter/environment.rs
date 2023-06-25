use anyhow::{bail, Result};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::inter::{ast::Expression, evaluator::EvaluationError};

use super::object::Object;

/// Represents an environment that stores variables and their corresponding values.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    /// Creates a new environment with an empty store and no outer environment.
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    /// Creates a new environment with an empty store and a specified outer environment.
    pub fn new_with_outer(outer: Rc<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    /// Sets the arguments to the environment, matching them with the corresponding parameters.
    /// Returns an error if the number of arguments and parameters don't match.
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

    /// Retrieves the name of an identifier expression.
    /// Returns an error if the expression is not an identifier.
    fn get_name(&self, name: &Expression) -> Result<String> {
        let name = match name {
            Expression::Identifier(s) => s,
            _ => bail!(EvaluationError::ExpectedIdentifier {
                found: name.type_as_string()
            }),
        };
        Ok(name.to_string())
    }

    /// Retrieves the value of a variable with the given name from the environment.
    /// If the variable is not found in the current environment, it recursively looks for it in the outer environments.
    /// Returns an error if the variable is not found.
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

    /// Sets the value of a variable with the given name in the environment.
    /// Returns an error if the name is not a valid identifier.
    pub fn set(&mut self, name: &Expression, value: Object) -> Result<()> {
        let name = self.get_name(name)?;
        self.store.insert(name, value);
        Ok(())
    }
}
