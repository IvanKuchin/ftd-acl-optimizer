use std::path::PathBuf;

use crate::acp::rule::network_object::network_object_optimized::NetworkObjectOptimized;

#[derive(thiserror::Error, Debug)]
pub enum FileError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No rule found with name: {name}")]
    RuleEmpty { name: String },
}

pub fn is_filtered(line: &str) -> bool {
    line.contains("Object missing: ") || line.contains("")
}

fn read_file(fname: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let content: Vec<_> = std::fs::read_to_string(fname)?
        .lines()
        .filter(|line| !is_filtered(line))
        .map(|s| s.to_string())
        .collect();

    Ok(content)
}

pub fn get_rule_lines_from_file(
    fname: &PathBuf,
    rule_name: &str,
) -> Result<Vec<String>, FileError> {
    let content = read_file(fname)?;

    let rule_lines: Vec<_> = content
        .iter()
        .skip_while(|line| !line.contains(&format!("Rule: {}", rule_name)))
        .take_while(|line| {
            !line.contains("Rule: ") || line.contains(&format!("Rule: {}", rule_name))
        })
        .cloned()
        .collect();

    if rule_lines.is_empty() {
        return Err(FileError::RuleEmpty {
            name: rule_name.to_string(),
        });
    }

    Ok(rule_lines)
}

pub fn print_networks_report(
    src_networks_opt: &Option<NetworkObjectOptimized>,
    dst_networks_opt: &Option<NetworkObjectOptimized>,
) {
    if let Some(src_networks) = src_networks_opt {
        let nets = get_optimized_elements(src_networks);

        if !nets.is_empty() {
            println!("\n\t --- Source networks ---");
            for net in nets.iter() {
                println!("\t\t {}", net);
            }
        }
    }
    if let Some(dst_networks) = dst_networks_opt {
        let nets = get_optimized_elements(dst_networks);

        if !nets.is_empty() {
            println!("\n\t --- Destination networks ---");
            for net in nets.iter() {
                println!("\t\t {}", net);
            }
        }
    }
}

fn get_optimized_elements(network_object: &NetworkObjectOptimized) -> Vec<String> {
    let result = network_object
        .items()
        .iter()
        .map(|item| item.name())
        .filter(|s| s.contains("ADJOINS") || s.contains("SHADOW") || s.contains("OVERLAP"))
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    result
}
