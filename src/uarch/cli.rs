use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "Vondel Microarchitecture")]
#[command(version = "1.0")]
#[command(about = "The Vondel Microarchitecture")]
#[command(author, long_about = None)]
#[command(
    help_template = "{author-with-newline} {about-section}Version: {version} \n\n {usage-heading} {usage} \n {all-args} {tab}"
)]

pub struct UArchCli {
    /// The name of the file that contains the ram dump
    #[arg(long)]
    pub ram: String,

    /// The name of the file that contains to firmware
    #[arg(long)]
    pub rom: String,

    /// Show number of cycles to execute the program
    #[arg(short, long, default_value_t = true)]
    pub cycles: bool,
    // /// Show the state of ram after execution
    // #[arg(long, default_value_t = false)]
    // pub ram_dump: bool,
}
