use rules::Rules;
use std::path::PathBuf;

pub mod rules;

use clap::Parser;

mod args;

#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Fail to parse rule: {0}")]
    Rule(#[from] rules::rule::RuleError),
    #[error("Fail to parse rules: {0}")]
    Rules(#[from] rules::RulesError),
    #[error("No rule found with name: {name}")]
    RuleEmpty { name: String },
    #[error("No rules found")]
    RulesEmpty(),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

fn main() {
    let args = args::AppArgs::parse();

    match args.subcommand {
        args::SubCommand::Analyze(analyze_args) => {
            if let Some(rule) = analyze_args.rule {
                match analyze_rule(&analyze_args.file, &rule) {
                    Ok(()) => {}
                    Err(e) => println!("Analysis failed: {}", e),
                }
            } else {
                match analyze_policy(&analyze_args.file) {
                    Ok(()) => {}
                    Err(e) => println!("Analysis failed: {}", e),
                }
            }
        }
    }
}

fn is_filtered(line: &str) -> bool {
    line.contains("Object missing: ") || line.contains("")
}

fn analyze_rule(fname: &PathBuf, rule_name: &str) -> Result<(), MyError> {
    let rule_lines: Vec<_> = std::fs::read_to_string(fname)?
        .lines()
        .skip_while(|line| !line.contains(&format!("Rule: {}", rule_name)))
        .take_while(|line| {
            !line.contains("Rule: ") || line.contains(&format!("Rule: {}", rule_name))
        })
        .filter(|line| !is_filtered(line))
        .map(|s| s.to_string())
        .collect();

    let rules = Rules::try_from(rule_lines)?;

    if rules.is_empty() {
        return Err(MyError::RuleEmpty {
            name: rule_name.to_string(),
        });
    }

    println!("# of rules found: {}", rules.len());
    println!("rule capacity: {}", rules.capacity());
    todo!("Implement analysis");
}

fn analyze_policy(fname: &PathBuf) -> Result<(), MyError> {
    let rule_lines: Vec<_> = std::fs::read_to_string(fname)?
        .lines()
        .skip_while(|line| !line.contains("-[ Rule: "))
        .take_while(|line| !line.contains("=[ Advanced Settings ]="))
        .filter(|line| !is_filtered(line))
        .map(|s| s.to_string())
        .collect();

    let rules = Rules::try_from(rule_lines)?;

    if rules.is_empty() {
        return Err(MyError::RulesEmpty {});
    }

    println!("# of rules found: {}", rules.len());
    println!("rule capacity: {}", rules.capacity());
    todo!("Implement analysis");
}
