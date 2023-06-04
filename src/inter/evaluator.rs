use crate::inter::{ast::Program, environment::Environment};

use super::ast;
use super::object::Object;
use anyhow::Result;
use thiserror::Error;

mod custom;
pub mod rust;

const TRUE: Object = Object::Boolean(true);
const FALSE: Object = Object::Boolean(false);
const NULL: Object = Object::Null;

#[derive(Debug, Error, PartialEq)]
pub enum EvaluationError {
    #[error("Unexpected object to invert signal: '{obj}, object must be an Integer")]
    MissingIntegerToInvert { obj: &'static str },

    #[error("Mismatched types in infix expression: expected 'Integer' and 'Integer' for '{operator}', found '{left} and '{right}'")]
    MismatchedTypesInfix {
        left: &'static str,
        right: &'static str,
        operator: &'static str,
    },

    #[error("Unexpected boolean operator '{operator}' for 'Boolean' and 'Boolean', must be '==', '!=', '&&' or '||'")]
    UnallowedBooleanComparisonOperator { operator: &'static str },

    #[error("Identifier not found: '{identifier}'")]
    IdentifierNotFound { identifier: String },

    #[error("Expected Identifier Expression found: '{found}'")]
    ExpectedIdentifier { found: &'static str },

    #[error("Wrong number of arguments: found '{found}' expected '{expected}'")]
    WrongNumberOfArguments { found: usize, expected: usize },

    #[error("Expected function found: '{found}'")]
    NotAFunction { found: &'static str },
}

pub(crate) trait Evaluator {
    fn eval(&self, node: &Program, env: &mut Environment) -> Result<Object>;
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use crate::inter::{
        ast::expression::{InfixOp, PrefixOp},
        environment::Environment,
        tokens::TokenType,
    };

    use super::*;
    use ast::*;

    fn test_eval_program(eval: &dyn Evaluator, prog: &ast::Program) -> Result<Object> {
        let mut e = Environment::new();
        let res = eval.eval(prog, &mut e)?;
        Ok(res)
    }

    fn create_program(ast: Vec<ast::StatementType>) -> ast::Program {
        ast::Program {
            statements: ast,
            errors: vec![],
        }
    }

    #[test]
    fn eval_integer_expression() -> Result<()> {
        // 5
        // 10
        // -5
        // -10
        // 5 * 2 - 10
        // 5 + 2 * 10
        // 20 + 2*-10
        // 2 * (5 + 10)
        // (5 + 10 * 2 + 15 / 3) * 2 + -10
        let ast = vec![
            StatementType::Expression(Expression::Integer(5)),
            StatementType::Expression(Expression::Integer(10)),
            StatementType::Expression(Expression::Prefix {
                op: PrefixOp::Minus,
                right: Box::new(Expression::Integer(5)),
            }),
            StatementType::Expression(Expression::Prefix {
                op: PrefixOp::Minus,
                right: Box::new(Expression::Integer(10)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    op: InfixOp::Asterisk,
                    right: Box::new(Expression::Integer(2)),
                }),
                op: InfixOp::Plus,
                right: Box::new(Expression::Integer(10)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::Plus,
                right: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(2)),
                    op: InfixOp::Asterisk,
                    right: Box::new(Expression::Integer(10)),
                }),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(20)),
                op: InfixOp::Plus,
                right: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(2)),
                    op: InfixOp::Asterisk,
                    right: Box::new(Expression::Prefix {
                        op: PrefixOp::Minus,
                        right: Box::new(Expression::Integer(10)),
                    }),
                }),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(2)),
                op: InfixOp::Asterisk,
                right: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    op: InfixOp::Plus,
                    right: Box::new(Expression::Integer(10)),
                }),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(Expression::Infix {
                            left: Box::new(Expression::Integer(5)),
                            op: InfixOp::Plus,
                            right: Box::new(Expression::Infix {
                                left: Box::new(Expression::Integer(10)),
                                op: InfixOp::Asterisk,
                                right: Box::new(Expression::Integer(2)),
                            }),
                        }),
                        op: InfixOp::Plus,
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Integer(15)),
                            op: InfixOp::Slash,
                            right: Box::new(Expression::Integer(3)),
                        }),
                    }),
                    op: InfixOp::Asterisk,
                    right: Box::new(Expression::Integer(2)),
                }),
                op: InfixOp::Plus,
                right: Box::new(Expression::Prefix {
                    op: PrefixOp::Minus,
                    right: Box::new(Expression::Integer(10)),
                }),
            }),
        ];

        let results = vec![5, 10, -5, -10, 20, 25, 0, 30, 50];

        let rust_eval = rust::RustEvaluator::new();

        for (idx, node) in ast.into_iter().enumerate() {
            let rust_res = test_eval_program(&rust_eval, &create_program(vec![node]))?;
            assert_eq!(rust_res.inspect(), results[idx].to_string())
        }

        Ok(())
    }

    #[test]
    fn eval_boolean_expression() -> Result<()> {
        /* true
         * false
         * 1 < 2
         * 1 > 2
         * 1 < 1
         * 1 > 1
         * 1 == 1
         * 1 != 1
         * 1 == 2
         * 1 != 2
         * true == true
         * false == false
         * true == false
         * true != false
         * false != true
         */
        let ast = vec![
            StatementType::Expression(Expression::Boolean(true)),
            StatementType::Expression(Expression::Boolean(false)),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(1)),
                op: InfixOp::LessThan,
                right: Box::new(Expression::Integer(2)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(1)),
                op: InfixOp::GreaterThan,
                right: Box::new(Expression::Integer(2)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(1)),
                op: InfixOp::LessThan,
                right: Box::new(Expression::Integer(1)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(1)),
                op: InfixOp::GreaterThan,
                right: Box::new(Expression::Integer(1)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(1)),
                op: InfixOp::Equal,
                right: Box::new(Expression::Integer(1)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(1)),
                op: InfixOp::NotEqual,
                right: Box::new(Expression::Integer(1)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(1)),
                op: InfixOp::Equal,
                right: Box::new(Expression::Integer(2)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(1)),
                op: InfixOp::NotEqual,
                right: Box::new(Expression::Integer(2)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Boolean(true)),
                op: InfixOp::Equal,
                right: Box::new(Expression::Boolean(true)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Boolean(false)),
                op: InfixOp::Equal,
                right: Box::new(Expression::Boolean(false)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Boolean(true)),
                op: InfixOp::Equal,
                right: Box::new(Expression::Boolean(false)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Boolean(true)),
                op: InfixOp::NotEqual,
                right: Box::new(Expression::Boolean(false)),
            }),
            StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Boolean(false)),
                op: InfixOp::NotEqual,
                right: Box::new(Expression::Boolean(true)),
            }),
        ];

        let results = vec![
            true, false, true, false, false, false, true, false, false, true, true, true, false,
            true, true,
        ];
        let rust_eval = rust::RustEvaluator::new();

        for (idx, node) in ast.into_iter().enumerate() {
            let rust_res = test_eval_program(&rust_eval, &create_program(vec![node]))?;
            assert_eq!(rust_res.inspect(), results[idx].to_string())
        }

        Ok(())
    }

    #[test]
    fn eval_bang_operator() -> Result<()> {
        let ast = vec![
            StatementType::Expression(
                Expression::new_prefix(&TokenType::Bang, Expression::Boolean(true)).unwrap(),
            ),
            StatementType::Expression(
                Expression::new_prefix(&TokenType::Bang, Expression::Boolean(false)).unwrap(),
            ),
            StatementType::Expression(
                Expression::new_prefix(&TokenType::Bang, Expression::Integer(5)).unwrap(),
            ),
            StatementType::Expression(
                Expression::new_prefix(
                    &TokenType::Bang,
                    Expression::new_prefix(&TokenType::Bang, Expression::Boolean(true)).unwrap(),
                )
                .unwrap(),
            ),
            StatementType::Expression(
                Expression::new_prefix(
                    &TokenType::Bang,
                    Expression::new_prefix(&TokenType::Bang, Expression::Boolean(false)).unwrap(),
                )
                .unwrap(),
            ),
            StatementType::Expression(
                Expression::new_prefix(
                    &TokenType::Bang,
                    Expression::new_prefix(&TokenType::Bang, Expression::Integer(5)).unwrap(),
                )
                .unwrap(),
            ),
        ];

        let results = vec![false, true, false, true, false, true];
        let rust_eval = rust::RustEvaluator::new();

        for (idx, node) in ast.into_iter().enumerate() {
            let rust_res = test_eval_program(&rust_eval, &create_program(vec![node]))?;
            assert_eq!(rust_res.inspect(), results[idx].to_string())
        }

        Ok(())
    }

    #[test]
    fn eval_if_else_statements() -> Result<()> {
        /*
         * if (true) { 10 }
         * if (false) { 10 }
         * if (1) { 10 }
         * if (1 < 2) { 10 }
         * if (1 > 2) { 10 }
         * if (1 > 2) { 10 } else { 20 }
         * if (1 < 2) { 10 } else { 20 }
         */
        let ast = vec![
            StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Boolean(true)),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Integer(10),
                )])),
                alternative: None,
            }),
            StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Boolean(false)),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Integer(10),
                )])),
                alternative: None,
            }),
            StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Integer(1)),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Integer(10),
                )])),
                alternative: None,
            }),
            StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(1)),
                    op: InfixOp::LessThan,
                    right: Box::new(Expression::Integer(2)),
                }),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Integer(10),
                )])),
                alternative: None,
            }),
            StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(1)),
                    op: InfixOp::GreaterThan,
                    right: Box::new(Expression::Integer(2)),
                }),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Integer(10),
                )])),
                alternative: None,
            }),
            StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(1)),
                    op: InfixOp::GreaterThan,
                    right: Box::new(Expression::Integer(2)),
                }),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Integer(10),
                )])),
                alternative: Some(Box::new(StatementType::Block(vec![
                    StatementType::Expression(Expression::Integer(20)),
                ]))),
            }),
            StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(1)),
                    op: InfixOp::LessThan,
                    right: Box::new(Expression::Integer(2)),
                }),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Integer(10),
                )])),
                alternative: Some(Box::new(StatementType::Block(vec![
                    StatementType::Expression(Expression::Integer(20)),
                ]))),
            }),
        ];

        let results = vec![
            10.to_string(),
            NULL.to_string(),
            10.to_string(),
            10.to_string(),
            NULL.to_string(),
            20.to_string(),
            10.to_string(),
        ];

        let rust_eval = rust::RustEvaluator::new();

        for (idx, node) in ast.into_iter().enumerate() {
            let rust_res = test_eval_program(&rust_eval, &create_program(vec![node]))?;
            assert_eq!(rust_res.inspect(), results[idx].to_string())
        }

        Ok(())
    }

    #[test]
    fn eval_return_statements() -> Result<()> {
        /*
         * return 1;
         * return 2; 9;
         * return 2 * 3; 9;
         * 9; return 4 * 5; 9;
         * if (10 > 1) {
         *     if (10 > 1) {
         *         return 100;
         *     }
         * return 1;
         * }
         */
        let programs = vec![
            vec![StatementType::Expression(Expression::Integer(1))],
            vec![
                StatementType::Return(Expression::Integer(2)),
                StatementType::Expression(Expression::Integer(9)),
            ],
            vec![
                StatementType::Return(Expression::Infix {
                    left: Box::new(Expression::Integer(2)),
                    op: InfixOp::Asterisk,
                    right: Box::new(Expression::Integer(3)),
                }),
                StatementType::Expression(Expression::Integer(9)),
            ],
            vec![
                StatementType::Expression(Expression::Integer(9)),
                StatementType::Return(Expression::Infix {
                    left: Box::new(Expression::Integer(4)),
                    op: InfixOp::Asterisk,
                    right: Box::new(Expression::Integer(5)),
                }),
                StatementType::Expression(Expression::Integer(9)),
            ],
            vec![StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(10)),
                    op: InfixOp::GreaterThan,
                    right: Box::new(Expression::Integer(1)),
                }),
                consequence: Box::new(StatementType::Block(vec![
                    StatementType::Expression(Expression::If {
                        condition: Box::new(Expression::Infix {
                            left: Box::new(Expression::Integer(10)),
                            op: InfixOp::GreaterThan,
                            right: Box::new(Expression::Integer(1)),
                        }),
                        consequence: Box::new(StatementType::Block(vec![StatementType::Return(
                            Expression::Integer(100),
                        )])),
                        alternative: None,
                    }),
                    StatementType::Return(Expression::Integer(1)),
                ])),
                alternative: None,
            })],
        ];
        let results = vec![1, 2, 6, 20, 100];

        let rust_eval = rust::RustEvaluator::new();

        for (idx, p) in programs.into_iter().enumerate() {
            let res = test_eval_program(&rust_eval, &create_program(p))?;
            assert_eq!(res.inspect(), results[idx].to_string());
        }
        Ok(())
    }

    #[test]
    fn test_error_handling() -> Result<()> {
        /*
         * 5 + true;
         * 5 + true; 5;
         * -true;
         * true + false;
         * 5; true + false; 5;
         * if (10 > 1) { true * false; };
         * if (10 > 1) {
         *    if (10 > 1) {
         *    return true - false;
         *    }
         *    return 1;
         * }
         *  tubias;
         */
        let programs = vec![
            vec![StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Integer(5)),
                op: InfixOp::Plus,
                right: Box::new(Expression::Boolean(true)),
            })],
            vec![
                StatementType::Expression(Expression::Infix {
                    left: Box::new(Expression::Integer(5)),
                    op: InfixOp::Plus,
                    right: Box::new(Expression::Boolean(true)),
                }),
                StatementType::Expression(Expression::Integer(5)),
            ],
            vec![StatementType::Expression(Expression::Prefix {
                op: PrefixOp::Minus,
                right: Box::new(Expression::Boolean(true)),
            })],
            vec![StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Boolean(true)),
                op: InfixOp::Plus,
                right: Box::new(Expression::Boolean(false)),
            })],
            vec![
                StatementType::Expression(Expression::Integer(5)),
                StatementType::Expression(Expression::Infix {
                    left: Box::new(Expression::Boolean(true)),
                    op: InfixOp::Plus,
                    right: Box::new(Expression::Boolean(false)),
                }),
                StatementType::Expression(Expression::Integer(5)),
            ],
            vec![StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(10)),
                    op: InfixOp::GreaterThan,
                    right: Box::new(Expression::Integer(1)),
                }),
                consequence: Box::new(StatementType::Block(vec![StatementType::Expression(
                    Expression::Infix {
                        left: Box::new(Expression::Boolean(true)),
                        op: InfixOp::Asterisk,
                        right: Box::new(Expression::Boolean(false)),
                    },
                )])),
                alternative: None,
            })],
            vec![StatementType::Expression(Expression::If {
                condition: Box::new(Expression::Infix {
                    left: Box::new(Expression::Integer(10)),
                    op: InfixOp::GreaterThan,
                    right: Box::new(Expression::Integer(1)),
                }),
                consequence: Box::new(StatementType::Block(vec![
                    StatementType::Expression(Expression::If {
                        condition: Box::new(Expression::Infix {
                            left: Box::new(Expression::Integer(10)),
                            op: InfixOp::GreaterThan,
                            right: Box::new(Expression::Integer(1)),
                        }),
                        consequence: Box::new(StatementType::Block(vec![StatementType::Return(
                            Expression::Infix {
                                left: Box::new(Expression::Boolean(true)),
                                op: InfixOp::Minus,
                                right: Box::new(Expression::Boolean(false)),
                            },
                        )])),
                        alternative: None,
                    }),
                    StatementType::Return(Expression::Integer(1)),
                ])),
                alternative: None,
            })],
            vec![StatementType::Expression(Expression::Identifier(
                "tubias".to_string(),
            ))],
        ];

        let errors = vec![
            EvaluationError::MismatchedTypesInfix {
                left: Object::Integer(5).type_as_string(),
                right: Object::Boolean(true).type_as_string(),
                operator: InfixOp::Plus.type_as_string(),
            },
            EvaluationError::MismatchedTypesInfix {
                left: Object::Integer(5).type_as_string(),
                right: Object::Boolean(true).type_as_string(),
                operator: InfixOp::Plus.type_as_string(),
            },
            EvaluationError::MissingIntegerToInvert {
                obj: Object::Boolean(true).type_as_string(),
            },
            EvaluationError::UnallowedBooleanComparisonOperator {
                operator: InfixOp::Plus.type_as_string(),
            },
            EvaluationError::UnallowedBooleanComparisonOperator {
                operator: InfixOp::Plus.type_as_string(),
            },
            EvaluationError::UnallowedBooleanComparisonOperator {
                operator: InfixOp::Asterisk.type_as_string(),
            },
            EvaluationError::UnallowedBooleanComparisonOperator {
                operator: InfixOp::Minus.type_as_string(),
            },
            EvaluationError::IdentifierNotFound {
                identifier: "tubias".to_string(),
            },
        ];

        let rust_eval = rust::RustEvaluator::new();

        for (idx, ast) in programs.into_iter().enumerate() {
            let err = test_eval_program(&rust_eval, &create_program(ast)).unwrap_err();
            assert_eq!(err.to_string(), errors[idx].to_string());
        }

        Ok(())
    }

    #[test]
    fn eval_let_statements() -> Result<()> {
        /*
         * let a = 5; a;
         * let a = 5 * 5; a;
         * let a = 5; let b = a; b;
         * let a = 5; let b = a; let c = a + b + 5; c;
         */
        let programs = vec![
            vec![
                StatementType::Let {
                    name: Expression::Identifier("a".to_string()),
                    value: Expression::Integer(5),
                },
                StatementType::Expression(Expression::Identifier("a".to_string())),
            ],
            vec![
                StatementType::Let {
                    name: Expression::Identifier("a".to_string()),
                    value: Expression::Infix {
                        left: Box::new(Expression::Integer(5)),
                        op: InfixOp::Asterisk,
                        right: Box::new(Expression::Integer(5)),
                    },
                },
                StatementType::Expression(Expression::Identifier("a".to_string())),
            ],
            vec![
                StatementType::Let {
                    name: Expression::Identifier("a".to_string()),
                    value: Expression::Integer(5),
                },
                StatementType::Let {
                    name: Expression::Identifier("b".to_string()),
                    value: Expression::Identifier("a".to_string()),
                },
                StatementType::Expression(Expression::Identifier("b".to_string())),
            ],
            vec![
                StatementType::Let {
                    name: Expression::Identifier("a".to_string()),
                    value: Expression::Integer(5),
                },
                StatementType::Let {
                    name: Expression::Identifier("b".to_string()),
                    value: Expression::Identifier("a".to_string()),
                },
                StatementType::Let {
                    name: Expression::Identifier("c".to_string()),
                    value: Expression::Infix {
                        left: Box::new(Expression::Infix {
                            left: Box::new(Expression::Identifier("a".to_string())),
                            op: InfixOp::Plus,
                            right: Box::new(Expression::Identifier("b".to_string())),
                        }),
                        op: InfixOp::Plus,
                        right: Box::new(Expression::Integer(5)),
                    },
                },
                StatementType::Expression(Expression::Identifier("c".to_string())),
            ],
        ];
        let results = vec![5, 25, 5, 15];

        let rust_eval = rust::RustEvaluator::new();

        for (idx, ast) in programs.into_iter().enumerate() {
            let rust_res = test_eval_program(&rust_eval, &create_program(ast))?;
            assert_eq!(rust_res.inspect(), results[idx].to_string());
        }

        Ok(())
    }

    #[test]
    fn eval_function_object() -> Result<()> {
        //fn(x) { x + 2; };
        let ast = vec![StatementType::Expression(Expression::FunctionLiteral {
            parameters: vec![Expression::Identifier("x".to_string())],
            block: Box::new(StatementType::Block(vec![StatementType::Expression(
                Expression::Infix {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    op: InfixOp::Plus,
                    right: Box::new(Expression::Integer(2)),
                },
            )])),
        })];

        let expect = Object::Function {
            params: vec![Expression::Identifier("x".to_string())],
            body: StatementType::Block(vec![StatementType::Expression(Expression::Infix {
                left: Box::new(Expression::Identifier("x".to_string())),
                op: InfixOp::Plus,
                right: Box::new(Expression::Integer(2)),
            })]),
            env: Rc::new(RefCell::new(Environment::new())),
        };

        let rust_eval = rust::RustEvaluator::new();

        let rust_res = test_eval_program(&rust_eval, &create_program(ast))?;

        assert_eq!(rust_res, expect);

        Ok(())
    }

    #[test]
    fn eval_function_application() -> Result<()> {
        /*
         * let identity = fn(x) { x; }; identity(5);
         * let identity = fn(x) { return x; }; identity(5);
         * let double = fn(x) { x * 2; }; double(5);
         * let add = fn(x, y) { x + y; }; add(5, 5);
         * let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));
         * fn(x) { x; }(5)
         */
        let programs = vec![
            vec![
                StatementType::Let {
                    name: Expression::Identifier("identity".to_string()),
                    value: Expression::FunctionLiteral {
                        parameters: vec![Expression::Identifier("x".to_string())],
                        block: Box::new(StatementType::Block(vec![StatementType::Expression(
                            Expression::Identifier("x".to_string()),
                        )])),
                    },
                },
                StatementType::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier("identity".to_string())),
                    arguments: vec![Expression::Integer(5)],
                }),
            ],
            vec![
                StatementType::Let {
                    name: Expression::Identifier("identity".to_string()),
                    value: Expression::FunctionLiteral {
                        parameters: vec![Expression::Identifier("x".to_string())],
                        block: Box::new(StatementType::Block(vec![StatementType::Return(
                            Expression::Identifier("x".to_string()),
                        )])),
                    },
                },
                StatementType::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier("identity".to_string())),
                    arguments: vec![Expression::Integer(5)],
                }),
            ],
            vec![
                StatementType::Let {
                    name: Expression::Identifier("double".to_string()),
                    value: Expression::FunctionLiteral {
                        parameters: vec![Expression::Identifier("x".to_string())],
                        block: Box::new(StatementType::Block(vec![StatementType::Expression(
                            Expression::Infix {
                                left: Box::new(Expression::Identifier("x".to_string())),
                                op: InfixOp::Asterisk,
                                right: Box::new(Expression::Integer(2)),
                            },
                        )])),
                    },
                },
                StatementType::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier("double".to_string())),
                    arguments: vec![Expression::Integer(5)],
                }),
            ],
            vec![
                StatementType::Let {
                    name: Expression::Identifier("add".to_string()),
                    value: Expression::FunctionLiteral {
                        parameters: vec![
                            Expression::Identifier("x".to_string()),
                            Expression::Identifier("y".to_string()),
                        ],
                        block: Box::new(StatementType::Block(vec![StatementType::Expression(
                            Expression::Infix {
                                left: Box::new(Expression::Identifier("x".to_string())),
                                op: InfixOp::Plus,
                                right: Box::new(Expression::Identifier("y".to_string())),
                            },
                        )])),
                    },
                },
                StatementType::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier("add".to_string())),
                    arguments: vec![Expression::Integer(5), Expression::Integer(5)],
                }),
            ],
            vec![
                StatementType::Let {
                    name: Expression::Identifier("add".to_string()),
                    value: Expression::FunctionLiteral {
                        parameters: vec![
                            Expression::Identifier("x".to_string()),
                            Expression::Identifier("y".to_string()),
                        ],
                        block: Box::new(StatementType::Block(vec![StatementType::Expression(
                            Expression::Infix {
                                left: Box::new(Expression::Identifier("x".to_string())),
                                op: InfixOp::Plus,
                                right: Box::new(Expression::Identifier("y".to_string())),
                            },
                        )])),
                    },
                },
                StatementType::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier("add".to_string())),
                    arguments: vec![
                        Expression::Infix {
                            left: Box::new(Expression::Integer(5)),
                            op: InfixOp::Plus,
                            right: Box::new(Expression::Integer(5)),
                        },
                        Expression::Call {
                            function: Box::new(Expression::Identifier("add".to_string())),
                            arguments: vec![Expression::Integer(5), Expression::Integer(5)],
                        },
                    ],
                }),
            ],
            vec![StatementType::Expression(Expression::Call {
                function: Box::new(Expression::FunctionLiteral {
                    parameters: vec![Expression::Identifier("x".to_string())],
                    block: Box::new(StatementType::Block(vec![StatementType::Expression(
                        Expression::Identifier("x".to_string()),
                    )])),
                }),
                arguments: vec![Expression::Integer(5)],
            })],
        ];

        let results = vec![5, 5, 10, 10, 20, 5];

        let rust_eval = rust::RustEvaluator::new();

        for (idx, ast) in programs.into_iter().enumerate() {
            let rust_res = test_eval_program(&rust_eval, &create_program(ast))?;
            assert_eq!(rust_res.inspect(), results[idx].to_string());
        }

        Ok(())
    }

    #[test]
    fn eval_function_with_invalid_parameters() -> Result<()> {
        /*
         * let addTwo(a,b){ return a + b; }; addTwo();
         * let addTwo(a,b){ return a + b; }; addTwo(1);
         */
        let programs = vec![
            vec![
                StatementType::Let {
                    name: Expression::Identifier("addTwo".to_string()),
                    value: Expression::FunctionLiteral {
                        parameters: vec![
                            Expression::Identifier("a".to_string()),
                            Expression::Identifier("b".to_string()),
                        ],
                        block: Box::new(StatementType::Block(vec![StatementType::Return(
                            Expression::Infix {
                                left: Box::new(Expression::Identifier("a".to_string())),
                                op: InfixOp::Plus,
                                right: Box::new(Expression::Identifier("b".to_string())),
                            },
                        )])),
                    },
                },
                StatementType::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier("addTwo".to_string())),
                    arguments: vec![],
                }),
            ],
            vec![
                StatementType::Let {
                    name: Expression::Identifier("addTwo".to_string()),
                    value: Expression::FunctionLiteral {
                        parameters: vec![
                            Expression::Identifier("a".to_string()),
                            Expression::Identifier("b".to_string()),
                        ],
                        block: Box::new(StatementType::Block(vec![StatementType::Return(
                            Expression::Infix {
                                left: Box::new(Expression::Identifier("a".to_string())),
                                op: InfixOp::Plus,
                                right: Box::new(Expression::Identifier("b".to_string())),
                            },
                        )])),
                    },
                },
                StatementType::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier("addTwo".to_string())),
                    arguments: vec![Expression::Integer(1)],
                }),
            ],
        ];

        let errors = vec![
            EvaluationError::WrongNumberOfArguments {
                expected: 2,
                found: 0,
            },
            EvaluationError::WrongNumberOfArguments {
                expected: 2,
                found: 1,
            },
        ];

        let rust_eval = rust::RustEvaluator::new();

        for (idx, ast) in programs.into_iter().enumerate() {
            let err = test_eval_program(&rust_eval, &create_program(ast)).unwrap_err();
            assert_eq!(err.to_string(), errors[idx].to_string());
        }

        Ok(())
    }

    #[test]
    fn eval_recursion() -> Result<()> {
        /*
         * let factorial = fn(n) { if (n == 0) { 1 } else { n * factorial(n - 1) } }; factorial(5);
         * let fibonacci = fn(x) { if (x == 0) { 0 } else { if (x == 1) { 1 } else { fibonacci(x - 1) + fibonacci(x - 2) } } }; fibonacci(10);
         */
        let programs = vec![vec![
            StatementType::Let {
                name: Expression::Identifier("factorial".to_string()),
                value: Expression::FunctionLiteral {
                    parameters: vec![Expression::Identifier("n".to_string())],
                    block: Box::new(StatementType::Block(vec![StatementType::Expression(
                        Expression::If {
                            condition: Box::new(Expression::Infix {
                                left: Box::new(Expression::Identifier("n".to_string())),
                                op: InfixOp::Equal,
                                right: Box::new(Expression::Integer(0)),
                            }),
                            consequence: Box::new(StatementType::Block(vec![
                                StatementType::Expression(Expression::Integer(1)),
                            ])),
                            alternative: Some(Box::new(StatementType::Block(vec![
                                StatementType::Expression(Expression::Infix {
                                    left: Box::new(Expression::Identifier("n".to_string())),
                                    op: InfixOp::Asterisk,
                                    right: Box::new(Expression::Call {
                                        function: Box::new(Expression::Identifier(
                                            "factorial".to_string(),
                                        )),
                                        arguments: vec![Expression::Infix {
                                            left: Box::new(Expression::Identifier("n".to_string())),
                                            op: InfixOp::Minus,
                                            right: Box::new(Expression::Integer(1)),
                                        }],
                                    }),
                                }),
                            ]))),
                        },
                    )])),
                },
            },
            StatementType::Expression(Expression::Call {
                function: Box::new(Expression::Identifier("factorial".to_string())),
                arguments: vec![Expression::Integer(5)],
            }),
        ]];
        let results = vec![120];

        let rust_eval = rust::RustEvaluator::new();

        for (idx, ast) in programs.into_iter().enumerate() {
            let rust_res = test_eval_program(&rust_eval, &create_program(ast))?;
            assert_eq!(rust_res.inspect(), results[idx].to_string());
        }

        Ok(())
    }
}
