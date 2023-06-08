use clap::{Parser, ValueEnum};

#[derive(Debug, ValueEnum, Clone)]
pub enum EvaluatorChoice {
    /// Use the Rust evaluator for processing the AST
    Rust,
    /// Use the Custom Evaluator for processing the AST
    Custom,
}

#[derive(Debug, Parser)]
#[command(name = "Vondel Interpreter")]
#[command(version = "1.0")]
#[command(about = "A simple interpreter for the Vondel Language")]
#[command(author, long_about = None)]
#[command(
    help_template = "{author-with-newline} {about-section}Version: {version} \n\n {usage-heading} {usage} \n {all-args} {tab}"
)]
pub struct InterpreterCli {
    /// Which evaluator to use with the Vondel Language
    #[arg(value_enum)]
    #[arg(short, long, default_value = "rust")]
    pub evaluator: EvaluatorChoice,

    /// The file to run the interpreter on
    #[arg(short, long)]
    pub file: Option<String>,
}
