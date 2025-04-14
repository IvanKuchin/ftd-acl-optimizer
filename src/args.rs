use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version, about, author)]
pub struct AppArgs {
    #[clap(subcommand)]
    /// Command to run
    pub subcommand: SubCommand,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    /// Analyze a rule or whole access policy from "show access-control-config"
    Analyze(AnalyzeArgs),
}

#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// Output of "show access-control-config"
    #[arg(short, long, required = true)]
    pub file: PathBuf,
    /// Rule name to analyze
    #[arg(short, long)]
    pub rule: Option<String>,
}
