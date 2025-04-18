use std::path::PathBuf;

use crate::acp::rule::network_object::network_object_optimized::NetworkObjectOptimized;

#[derive(thiserror::Error, Debug)]
pub enum FileError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No rule found with name: {name}")]
    RuleEmpty { name: String },
    #[error("No access control policy found in file: {file}")]
    AcpEmpty { file: String },
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

pub fn read_acp_from_file(fname: &PathBuf) -> Result<Vec<String>, FileError> {
    let content = read_file(fname)?;

    let acp: Vec<_> = content
        .iter()
        .skip_while(|line| !line.contains("--[ Rule: "))
        .take_while(|line| !line.contains("==[ Advanced Settings ]=="))
        .cloned()
        .collect();

    if acp.is_empty() {
        return Err(FileError::AcpEmpty {
            file: fname.to_string_lossy().to_string(),
        });
    }

    Ok(acp)
}

pub fn print_optimization_report(
    src_networks_opt: &Option<NetworkObjectOptimized>,
    dst_networks_opt: &Option<NetworkObjectOptimized>,
) {
    if let Some(src_networks) = src_networks_opt {
        let nets = get_optimized_elements_name(src_networks);

        if !nets.is_empty() {
            println!("\n\t --- {} ---", src_networks.name());
            for net in nets.iter() {
                println!("\t\t {}", net);
            }
        }
    }
    if let Some(dst_networks) = dst_networks_opt {
        let nets = get_optimized_elements_name(dst_networks);

        if !nets.is_empty() {
            println!("\n\t --- {} ---", dst_networks.name());
            for net in nets.iter() {
                println!("\t\t {}", net);
            }
        }
    }
}

fn get_optimized_elements_name(network_object: &NetworkObjectOptimized) -> Vec<String> {
    let result = network_object
        .items()
        .iter()
        .map(|item| item.name())
        .filter(|s| s.contains("ADJOINS") || s.contains("SHADOW") || s.contains("OVERLAP"))
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    result
}
