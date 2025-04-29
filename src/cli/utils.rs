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

fn is_filtered(line: &str) -> bool {
    line.contains("Object missing: ") || line.contains("")
}

/// Checks if a line contains an open parenthesis without a corresponding close parenthesis.
/// This function is used to identify lines that start a parenthetical block but do not
/// complete it, which is useful for merging multiline entries.
fn is_open_parenthesis(line: &str) -> bool {
    line.contains('(') && !line.contains(')')
}

/// Checks if a line contains a closing parenthesis `)` without an opening parenthesis `(`.
/// # Arguments
/// * `line` - A string slice representing the line to check.
///
/// # Returns
/// * `true` if the line contains a closing parenthesis `)` but no opening parenthesis `(`.
/// * `false` otherwise.
fn is_close_parenthesis(line: &str) -> bool {
    line.contains(')') && !line.contains('(')
}

/// Read a file and merge lines that are part of the same entry.
/// For example:
///  OBJ-10.223.149.185-198 (10.223.149.185-10.223.149.
///  198)
/// Should be merged to:
///  OBJ-10.223.149.185-198 (10.223.149.185-10.223.149.198)
fn merge_lines_between_parenthesis<'a>(iter: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    let mut in_parenthesis = false;
    for line in iter {
        if in_parenthesis {
            if let Some(last_line) = result.last_mut() {
                last_line.push_str(line);
            }
            if is_close_parenthesis(line) {
                in_parenthesis = false;
            }
            continue;
        }
        if is_open_parenthesis(line) {
            in_parenthesis = true;
        }
        // Add the line as a new entry
        result.push(line.to_string());
    }

    result
}

pub fn read_and_merge_lines(fname: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let content = std::fs::read_to_string(fname)?;

    let result = merge_lines_between_parenthesis(content.lines());

    Ok(result)
}

fn read_file(fname: &PathBuf) -> Result<Vec<String>, std::io::Error> {
    let content: Vec<_> = read_and_merge_lines(fname)?
        .into_iter()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_lines_basic_merge() {
        let input = r#"OBJ-10.223.149.185-198 (10.223.149.185-10.223.149.
198)
Another line"#;
        let expected = vec![
            "OBJ-10.223.149.185-198 (10.223.149.185-10.223.149.198)",
            "Another line",
        ];

        let result = merge_lines_between_parenthesis(input.lines());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_lines_no_merge() {
        let input = vec!["Line 1", "Line 2", "Line 3"];
        let expected = vec!["Line 1", "Line 2", "Line 3"];

        let result = merge_lines_between_parenthesis(input.into_iter());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_lines_multiple_merges_1() {
        let input = vec![
            "OBJ-10.223.149.185-198 (10.223.149.",
            "185-10.223.",
            "149.198)",
            "Another line",
        ];
        let expected = vec![
            "OBJ-10.223.149.185-198 (10.223.149.185-10.223.149.198)",
            "Another line",
        ];

        let result = merge_lines_between_parenthesis(input.into_iter());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_lines_multiple_merges_2() {
        let input = vec![
            "    Source Networks       : range-10.220.240.100-124 (10.220.240.100-10.",
            "220.240.124)",
            "range-10.220.240.209-238 (10.220.240.209-10.220.240.23",
            "8)",
            "range-10.217.240.112-136 (10.217.240.112-10.217.240.13",
            "6)",
            "range-10.217.241.1-153 (10.217.241.1-10.217.241.153)",
            "Another line",
        ];
        let expected = vec![
            "    Source Networks       : range-10.220.240.100-124 (10.220.240.100-10.220.240.124)",
            "range-10.220.240.209-238 (10.220.240.209-10.220.240.238)",
            "range-10.217.240.112-136 (10.217.240.112-10.217.240.136)",
            "range-10.217.241.1-153 (10.217.241.1-10.217.241.153)",
            "Another line",
        ];

        let result = merge_lines_between_parenthesis(input.into_iter());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_lines_empty_input() {
        let input: Vec<&str> = vec![];
        let expected: Vec<String> = vec![];

        let result = merge_lines_between_parenthesis(input.into_iter());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_merge_lines_no_open_parenthesis_special_characters() {
        let input = vec!["Line with special chars: @#$%", "123.456)", "Another line"];
        let expected = vec!["Line with special chars: @#$%", "123.456)", "Another line"];

        let result = merge_lines_between_parenthesis(input.into_iter());
        assert_eq!(result, expected);
    }
}
