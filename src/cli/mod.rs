use crate::acp::{rule, Acp};
use std::path::PathBuf;

mod utils;

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Fail to parse rule: {0}")]
    Rule(#[from] crate::acp::rule::RuleError),
    #[error("No rule found with name: {name}")]
    RuleEmpty { name: String },
    #[error("Fail to parse rules: {0}")]
    Acp(#[from] crate::acp::AcpError),
    #[error("No rules found")]
    AcpEmpty(),

    #[error("CLI parsing error: {0}")]
    Cli(#[from] utils::FileError),
}

fn analyze_rule(fname: &PathBuf, rule_name: &str) -> Result<(), CliError> {
    let rule_lines = utils::get_rule(fname, rule_name)?;

    let rules = Acp::try_from(rule_lines)?;

    if rules.is_empty() {
        return Err(CliError::RuleEmpty {
            name: rule_name.to_string(),
        });
    }

    println!("# of rules found: {}", rules.len());
    println!("rule capacity: {}", rules.capacity());
    todo!("Implement analysis");
}

pub fn analyze_rule_capacity(fname: &PathBuf, rule_name: &str) -> Result<(), CliError> {
    let rule_lines = utils::get_rule(fname, rule_name)?;

    let acp = Acp::try_from(rule_lines)?;

    if acp.is_empty() {
        return Err(CliError::RuleEmpty {
            name: rule_name.to_string(),
        });
    }

    let rule = acp.rule_by_name(rule_name).ok_or(CliError::RuleEmpty {
        name: rule_name.to_string(),
    })?;

    println!("Rule name: {}", rule.get_name());

    println!("\t capacity:           {}", rule.capacity());
    println!("\t optimized capacity: {}", rule.optimized_capacity());

    println!(
        "\t optimization ratio: {:.2}%",
        100. - (rule.optimized_capacity() as f64 / rule.capacity() as f64) * 100.0
    );

    Ok(())
}

fn analyze_policy(fname: &PathBuf) -> Result<(), CliError> {
    let rule_lines: Vec<_> = std::fs::read_to_string(fname)?
        .lines()
        .skip_while(|line| !line.contains("-[ Rule: "))
        .take_while(|line| !line.contains("=[ Advanced Settings ]="))
        .filter(|line| !utils::is_filtered(line))
        .map(|s| s.to_string())
        .collect();

    let acp = Acp::try_from(rule_lines)?;

    if acp.is_empty() {
        return Err(CliError::AcpEmpty {});
    }

    println!("# of rules found: {}", acp.len());
    println!("rule capacity: {}", acp.capacity());
    todo!("Implement analysis");
}
