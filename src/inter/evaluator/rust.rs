use std::{cell::RefCell, rc::Rc};

use crate::inter::{
    ast::expression::{InfixOp, PrefixOp},
    environment::Environment,
    object::Object,
};

use super::ast::*;
use anyhow::{bail, Result};

/// Represents an evaluator for Rust code.
pub struct RustEvaluator {}

impl RustEvaluator {
    /// Creates a new instance of `RustEvaluator`.
    pub fn new() -> Self {
        RustEvaluator {}
    }

    /// Maps a boolean value to the corresponding `Object`.
    fn map_boolean(&self, b: bool) -> Object {
        if b {
            super::TRUE
        } else {
            super::FALSE
        }
    }

    /// Evaluates the logical NOT (`!`) operator on the right operand.
    fn eval_bang_operator(&self, right: Object) -> Result<Object> {
        match right {
            Object::Boolean(true) => Ok(super::FALSE),
            Object::Boolean(false) => Ok(super::TRUE),
            Object::Null => Ok(super::TRUE),
            _ => Ok(super::FALSE),
        }
    }

    /// Evaluates the arithmetic minus (`-`) operator on the right operand.
    fn eval_minus_operator(&self, right: Object) -> Result<Object> {
        match right {
            Object::Integer(i) => Ok(Object::Integer(-i)),
            _ => bail!(super::EvaluationError::MissingIntegerToInvert {
                obj: right.type_as_string()
            }),
        }
    }

    /// Evaluates a prefix expression (e.g., -, !)
    fn eval_prefix_expression(
        &self,
        op: &PrefixOp,
        right: &Expression,
        e: &mut Environment,
    ) -> Result<Object> {
        let right = self.eval_expression(right, e)?;
        match *op {
            PrefixOp::Bang => self.eval_bang_operator(right),
            PrefixOp::Minus => self.eval_minus_operator(right),
        }
    }

    /// Evaluates an infix expression (e.g., +, -, *, /, <, >, ==, !=).
    fn eval_infix_expression(
        &self,
        left: &Expression,
        op: &InfixOp,
        right: &Expression,
        e: &mut Environment,
    ) -> Result<Object> {
        let left = self.eval_expression(left, e)?;
        let right = self.eval_expression(right, e)?;

        match (&left, &right, &op) {
            (Object::Integer(l), Object::Integer(r), o) => match o {
                InfixOp::Plus => Ok(Object::Integer(l + r)),
                InfixOp::Minus => Ok(Object::Integer(l - r)),
                InfixOp::Asterisk => Ok(Object::Integer(l * r)),
                InfixOp::Slash => Ok(Object::Integer(l / r)),
                InfixOp::LessThan => Ok(self.map_boolean(l < r)),
                InfixOp::GreaterThan => Ok(self.map_boolean(l > r)),
                InfixOp::Equal => Ok(self.map_boolean(l == r)),
                InfixOp::NotEqual => Ok(self.map_boolean(l != r)),
            },
            (Object::Boolean(l), Object::Boolean(r), o) => match o {
                InfixOp::Equal => Ok(self.map_boolean(l == r)),
                InfixOp::NotEqual => Ok(self.map_boolean(l != r)),
                _ => bail!(super::EvaluationError::UnallowedBooleanComparisonOperator {
                    operator: op.type_as_string(),
                }),
            },
            _ => bail!(super::EvaluationError::MismatchedTypesInfix {
                left: left.type_as_string(),
                right: right.type_as_string(),
                operator: op.type_as_string(),
            }),
        }
    }

    /// Checks if an object is truthy (evaluates to true).
    fn is_truthy(&self, obj: &Object) -> bool {
        match obj {
            Object::Null => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }

    /// Evaluates an `if` expression.
    fn eval_if_expression(
        &self,
        cond: &Expression,
        cons: &StatementType,
        alt: &Option<Box<StatementType>>,
        e: &mut Environment,
    ) -> Result<Object> {
        let condition = self.eval_expression(cond, e)?;
        match (self.is_truthy(&condition), alt) {
            (true, _) => Ok(self.eval_statements(cons, e)?),
            (false, Some(a)) => Ok(self.eval_statements(a, e)?),
            (false, None) => Ok(super::NULL),
        }
    }

    /// Evaluates a `let` statement.
    fn eval_let_statement(
        &self,
        name: &Expression,
        value: &Expression,
        e: &mut Environment,
    ) -> Result<Object> {
        let value = self.eval_expression(value, e)?;
        e.set(name, value.clone())?;
        Ok(value)
    }

    /// Evaluates expressions in a given environment.
    fn eval_expressions(&self, params: &[Expression], e: &mut Environment) -> Result<Vec<Object>> {
        let mut args = Vec::with_capacity(params.len());
        for p in params {
            let p = self.eval_expression(p, e)?;
            args.push(p);
        }
        Ok(args)
    }

    /// Evaluates a function call expression.
    fn eval_fn_call(
        &self,
        function: &Expression,
        arguments: &[Expression],
        e: &mut Environment,
    ) -> Result<Object> {
        let func = self.eval_expression(function, e)?;

        match func {
            Object::Function {
                params,
                body,
                env: f_env,
            } => {
                let args = self.eval_expressions(arguments, e)?;

                if args.len() != params.len() {
                    bail!(super::EvaluationError::WrongNumberOfArguments {
                        expected: params.len(),
                        found: args.len(),
                    });
                };

                let mut extended_env = Environment::new_with_outer(Rc::clone(&f_env));
                extended_env.set_arguments_to_env(args, params.clone())?;

                if let Expression::Identifier(_) = function {
                    extended_env.set(
                        function,
                        Object::Function {
                            params,
                            body: body.clone(),
                            env: f_env,
                        },
                    )?;
                }

                let evaluated = self.eval_statements(&body, &mut extended_env)?;

                if let Object::ReturnValue(v) = evaluated {
                    Ok(*v)
                } else {
                    Ok(evaluated)
                }
            }
            _ => bail!(super::EvaluationError::NotAFunction {
                found: func.type_as_string(),
            }),
        }
    }

    /// Evaluates an expression.
    fn eval_expression(&self, expr: &Expression, e: &mut Environment) -> Result<Object> {
        match *expr {
            Expression::Integer(i) => Ok(Object::Integer(i)),
            Expression::Boolean(b) => Ok(self.map_boolean(b)),
            Expression::Prefix { ref op, ref right } => {
                Ok(self.eval_prefix_expression(op, right, e)?)
            }
            Expression::Infix {
                ref left,
                ref op,
                ref right,
            } => Ok(self.eval_infix_expression(left, op, right, e)?),
            Expression::If {
                ref condition,
                ref consequence,
                ref alternative,
            } => Ok(self.eval_if_expression(condition, consequence, alternative, e)?),
            Expression::Identifier(ref name) => Ok(e.get(name)?),
            Expression::FunctionLiteral {
                ref parameters,
                ref block,
            } => Ok(Object::Function {
                params: parameters.clone(),
                body: *block.clone(),
                env: Rc::new(RefCell::new(e.clone())),
            }),
            Expression::Call {
                ref function,
                ref arguments,
            } => self.eval_fn_call(function, arguments, e),
        }
    }

    /// Evaluates a block of statements in a given environment.
    fn eval_block_statements(
        &self,
        stmts: &[super::ast::StatementType],
        e: &mut Environment,
    ) -> Result<Object> {
        let mut res = Object::Null;
        for stmt in stmts.iter() {
            res = self.eval_statements(stmt, e)?;
            if let Object::ReturnValue(_) = res {
                return Ok(res);
            }
        }
        Ok(res)
    }

    /// Evaluates statements in a given environment.
    fn eval_statements(
        &self,
        node: &super::ast::StatementType,
        e: &mut Environment,
    ) -> Result<super::Object> {
        match *node {
            StatementType::Expression(ref expr) => self.eval_expression(expr, e),
            StatementType::Block(ref block) => self.eval_block_statements(block, e),
            StatementType::Return(ref expr) => Ok(Object::ReturnValue(Box::new(
                self.eval_expression(expr, e)?,
            ))),
            StatementType::Let {
                ref name,
                ref value,
            } => self.eval_let_statement(name, value, e),
        }
    }
}

impl super::Evaluator for RustEvaluator {
    /// Evaluates a program in a given environment.
    fn eval(&self, prog: &Program, e: &mut Environment) -> Result<super::Object> {
        let mut res = Object::Null;
        for stmt in prog.statements.iter() {
            res = self.eval_statements(stmt, e)?;
            if let Object::ReturnValue(v) = res {
                return Ok(*v);
            }
        }
        Ok(res)
    }
}
