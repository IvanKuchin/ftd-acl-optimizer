use std::path::PathBuf;

use crate::acp::Acp;

pub mod args;
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
    #[error("Fail to parse access control policy: {0}")]
    Acp(#[from] crate::acp::AcpError),

    #[error("CLI parsing error: {0}")]
    Cli(#[from] utils::FileError),
}

fn get_acp(fname: &PathBuf) -> Result<Acp, CliError> {
    let rule_lines = utils::read_acp_from_file(fname)?;

    let acp = Acp::try_from(rule_lines)?;

    if acp.is_empty() {
        return Err(CliError::AcpEmpty {
            file: fname.to_string_lossy().to_string(),
        });
    }

    Ok(acp)
}

pub fn analyze_rule(fname: &PathBuf, rule_name: &str) -> Result<(), CliError> {
    let acp = get_acp(fname)?;

    let rule = acp.rule_by_name(rule_name).ok_or(CliError::RuleEmpty {
        name: rule_name.to_string(),
    })?;

    let rule_capacity = rule.capacity();
    let rule_capacity_optimized = rule.optimized_capacity();

    utils::print_rule_analysis(rule.get_name(), rule_capacity, rule_capacity_optimized);

    let (src_networks_opt, dst_networks_opt) = rule.get_optimized_networks();
    utils::print_optimization_report(&src_networks_opt, &dst_networks_opt);

    Ok(())
}

pub fn analyze_rule_capacity(fname: &PathBuf, rule_name: &str) -> Result<(), CliError> {
    let acp = get_acp(fname)?;

    let rule = acp.rule_by_name(rule_name).ok_or(CliError::RuleEmpty {
        name: rule_name.to_string(),
    })?;

    utils::print_rule_analysis(rule.get_name(), rule.capacity(), rule.optimized_capacity());

    Ok(())
}

pub fn analyze_acp_capacity(fname: &PathBuf) -> Result<(), CliError> {
    let acp = get_acp(fname)?;
    let mut acp_capacity: u64 = 0;
    let mut acp_capacity_optimized: u64 = 0;

    println!("==== Rules analysis ====");
    for rule in acp.iter() {
        let rule_capacity = rule.capacity();
        let rule_capacity_optimized = rule.optimized_capacity();
        acp_capacity += rule_capacity;
        acp_capacity_optimized += rule_capacity_optimized;

        utils::print_rule_analysis(rule.get_name(), rule_capacity, rule_capacity_optimized);
    }

    println!("\n");
    println!("==== Access Control Policy ====");
    println!("# of rules found: {}", acp.len());
    println!("acp capacity: {}", acp_capacity);
    println!("acp optimized capacity: {}", acp_capacity_optimized);
    println!(
        "acp optimization ratio: {:.2}%",
        100. - (acp_capacity_optimized as f64 / acp_capacity as f64) * 100.0
    );

    Ok(())
}

pub fn analyze_acp(fname: &PathBuf) -> Result<(), CliError> {
    let acp = get_acp(fname)?;
    let mut acp_capacity: u64 = 0;
    let mut acp_capacity_optimized: u64 = 0;

    println!("==== Rules analysis ====");
    for rule in acp.iter() {
        let rule_capacity = rule.capacity();
        let rule_capacity_optimized = rule.optimized_capacity();
        acp_capacity += rule_capacity;
        acp_capacity_optimized += rule_capacity_optimized;

        utils::print_rule_analysis(rule.get_name(), rule_capacity, rule_capacity_optimized);

        let (src_networks_opt, dst_networks_opt) = rule.get_optimized_networks();
        utils::print_optimization_report(&src_networks_opt, &dst_networks_opt);
    }

    println!("\n");
    println!("==== Access Control Policy ====");
    println!("# of rules found: {}", acp.len());
    println!("acp capacity: {}", acp_capacity);
    println!("acp optimized capacity: {}", acp_capacity_optimized);
    println!(
        "acp optimization ratio: {:.2}%",
        100. - (acp_capacity_optimized as f64 / acp_capacity as f64) * 100.0
    );

    Ok(())
}

pub fn analyze_topk_by_capacity(fname: &PathBuf, k: usize) -> Result<(), CliError> {
    let acp = get_acp(fname)?;

    let mut rules = acp.iter().collect::<Vec<_>>();

    rules.sort_by_key(|a| a.capacity());
    rules.reverse();

    println!("==== Top{k} rules by capacity ====");
    for rule in rules.iter().take(k) {
        let rule_capacity = rule.capacity();
        let rule_capacity_optimized = rule.optimized_capacity();

        utils::print_rule_analysis(rule.get_name(), rule_capacity, rule_capacity_optimized);
    }

    Ok(())
}

pub fn analyze_topk_by_optimization(fname: &PathBuf, k: usize) -> Result<(), CliError> {
    let acp = get_acp(fname)?;

    let mut rules = acp.iter().collect::<Vec<_>>();

    rules.sort_by_key(|a| a.capacity() - a.optimized_capacity());
    rules.reverse();

    println!("==== Top{k} rules by capacity ====");
    for rule in rules.iter().take(k) {
        let rule_capacity = rule.capacity();
        let rule_capacity_optimized = rule.optimized_capacity();

        utils::print_rule_analysis(rule.get_name(), rule_capacity, rule_capacity_optimized);
    }

    Ok(())
}
