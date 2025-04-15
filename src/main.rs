use clap::Parser;
use std::path::PathBuf;

pub mod acp;

mod args;
mod cli;

fn main() {
    let args = args::AppArgs::parse();
    let file = args.file;

    match args.subcommand {
        args::Verb::Get(entity) => match entity {
            args::Entity::Rule(rule) => parse_rule(&file, rule),
            args::Entity::TopK(topk) => parse_topk(&file, topk),
            args::Entity::Acp(acp) => parse_acp(&file, acp),
        },
    }
}

fn parse_rule(file: &PathBuf, rule: args::Rule) {
    match rule {
        args::Rule::Capacity(rule_name) => {
            cli::analyze_rule_capacity(file, &rule_name.name).unwrap_or_else(|e| {
                eprintln!("Error analyzing rule capacity: {}", e);
            });
        }
        args::Rule::RuleAnalysis(rule_name) => {
            println!("Analyzing rule: {}", rule_name.name);
        }
    }
}

fn parse_topk(file: &PathBuf, topk: args::TopK) {
    todo!("Analyzing top-k rules from file: {}", file.display());
}

fn parse_acp(file: &PathBuf, acp: args::Acp) {
    todo!(
        "Analyzing the whole access policy from file: {}",
        file.display()
    );
}
