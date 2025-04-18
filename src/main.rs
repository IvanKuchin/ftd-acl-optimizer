use clap::Parser;
use std::path::PathBuf;

pub mod acp;

mod args;
mod cli;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Fail to run app due to rule analysis error: {0}")]
    Rule(#[from] cli::CliError),
}

fn main() {
    let args = args::AppArgs::parse();
    let file = args.file;

    let err = match args.subcommand {
        args::Verb::Get(entity) => match entity {
            args::Entity::Rule(rule) => parse_rule(&file, rule),
            args::Entity::TopK(topk) => parse_topk(&file, topk),
            args::Entity::Acp(acp) => parse_acp(&file, acp),
        },
    };

    if let Err(e) = err {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn parse_rule(file: &PathBuf, action: args::Rule) -> Result<(), AppError> {
    match action {
        args::Rule::Capacity(rule_name) => cli::analyze_rule_capacity(file, &rule_name.name)?,
        args::Rule::Analysis(rule_name) => cli::analyze_rule(file, &rule_name.name)?,
    };

    Ok(())
}

fn parse_topk(file: &PathBuf, action: args::TopK) -> Result<(), AppError> {
    match action {
        args::TopK::ByCapacity(nop) => cli::analyze_topk_by_capacity(file, 5)?,
        args::TopK::ByOptimization(nop) => cli::analyze_topk_by_optimization(file, 5)?,
    };

    Ok(())
}

fn parse_acp(file: &PathBuf, action: args::Acp) -> Result<(), AppError> {
    match action {
        args::Acp::Capacity(nop) => cli::analyze_acp_capacity(file)?,
        args::Acp::Analysis(nop) => cli::analyze_acp(file)?,
    };

    Ok(())
}
