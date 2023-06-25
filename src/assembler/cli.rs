use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "Vondel Assembler")]
#[command(version = "1.0")]
#[command(about = "A simple assembler for the Vondel Microarchitecture")]
#[command(author, long_about = None)]
#[command(
    help_template = "{author-with-newline} {about-section}Version: {version} \n\n {usage-heading} {usage} \n {all-args} {tab}"
)]

pub struct AssemblerCli {
    /// The name of the file to assemble
    #[arg(short, long)]
    pub input: String,

    /// The name of the output
    #[arg(short, long, default_value = "a")]
    pub output: Option<String>,
}
