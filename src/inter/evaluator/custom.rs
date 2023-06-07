use crate::inter::{ast::Program, environment::Environment, evaluator::Evaluator};
use anyhow::Result;

pub struct CustomEvaluator {}

impl CustomEvaluator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Evaluator for CustomEvaluator {
    fn eval(&self, _prog: &Program, _e: &mut Environment) -> Result<super::Object> {
        todo!();
    }
}
