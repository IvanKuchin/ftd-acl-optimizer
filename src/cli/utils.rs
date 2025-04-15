use std::path::PathBuf;

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

pub fn get_rule(fname: &PathBuf, rule_name: &str) -> Result<Vec<String>, FileError> {
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
