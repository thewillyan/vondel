use anyhow::Result;
use clap::Parser;
use vondel::inter::{
    cli::{EvaluatorChoice, InterpreterCli},
    evaluator::{self, evaluate_buffer},
    repl,
};
fn read_from_file(file: &str) -> Result<String> {
    let buf = match std::fs::read_to_string(file) {
        Ok(buf) => buf,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };
    Ok(buf)
}

fn main() -> Result<()> {
    let cli = InterpreterCli::parse();
    let evaluator: Box<dyn evaluator::Evaluator> = match cli.evaluator {
        EvaluatorChoice::Rust => Box::new(evaluator::rust::RustEvaluator::new()),
        EvaluatorChoice::Custom => Box::new(evaluator::custom::CustomEvaluator::new()),
    };

    match cli.file {
        Some(file) => {
            let buf = read_from_file(&file)?;
            evaluate_buffer(evaluator, buf)?;
        }
        None => repl::start(evaluator),
    }

    Ok(())
}
