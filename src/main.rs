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

fn parse_rule(file: &PathBuf, rule: args::Rule) -> Result<(), AppError> {
    match rule {
        args::Rule::Capacity(rule_name) => {
            cli::analyze_rule_capacity(file, &rule_name.name)?;
        }
        args::Rule::RuleAnalysis(rule_name) => {
            println!("Analyzing rule: {}", rule_name.name);
            todo!("Implement rule analysis for: {}", rule_name.name);
        }
    };

    Ok(())
}

fn parse_topk(file: &PathBuf, topk: args::TopK) -> Result<(), AppError> {
    todo!("Analyzing top-k rules from file: {}", file.display());
}

fn parse_acp(file: &PathBuf, acp: args::Acp) -> Result<(), AppError> {
    todo!(
        "Analyzing the whole access policy from file: {}",
        file.display()
    );
}
