use std::path::PathBuf;

use crate::acp::Acp;

mod utils;

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Fail to parse rule: {0}")]
    Rule(#[from] crate::acp::rule::RuleError),
    #[error("Can't find access-control-policy or it is empty ({file})")]
    AcpEmpty { file: String },
    #[error("No rule found with name ({name})")]
    RuleEmpty { name: String },
    #[error("Fail to parse rules: {0}")]
    Acp(#[from] crate::acp::AcpError),

    #[error("CLI parsing error: {0}")]
    Cli(#[from] utils::FileError),
}

fn get_acp_rule<'a>(fname: &PathBuf, rule_name: &'a str) -> Result<Acp, CliError> {
    let rule_lines = utils::get_rule_lines_from_file(fname, rule_name)?;

    let acp = Acp::try_from(rule_lines)?;

    if acp.is_empty() {
        return Err(CliError::AcpEmpty {
            file: fname.to_string_lossy().to_string(),
        });
    }

    Ok(acp)
}

pub fn analyze_rule(fname: &PathBuf, rule_name: &str) -> Result<(), CliError> {
    let acp = get_acp_rule(fname, rule_name)?;

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

    let (src_networks_opt, dst_networks_opt) = rule.get_optimized_networks();
    utils::print_networks_report(&src_networks_opt, &dst_networks_opt);

    Ok(())
}

pub fn analyze_rule_capacity(fname: &PathBuf, rule_name: &str) -> Result<(), CliError> {
    let acp = get_acp_rule(fname, rule_name)?;

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
        return Err(CliError::AcpEmpty {
            file: fname.to_string_lossy().to_string(),
        });
    }

    println!("# of rules found: {}", acp.len());
    println!("rule capacity: {}", acp.capacity());
    todo!("Implement analysis");
}
