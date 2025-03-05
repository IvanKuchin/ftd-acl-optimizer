#[derive(thiserror::Error, Debug)]
pub enum UtilitiesError {
    #[error("Fail to extract name: {0}")]
    NameExtractionError(String),
    #[error("Fail to extract name with details: {0}, {1}")]
    NameExtractionError2(String, String),
    #[error("Fail to calculate lines in a group: {0}")]
    GroupLineCalculationError(String),
    #[error("Fail to calculate lines in a group with details: {0}, {1}")]
    GroupLineCalculationError2(String, String),
}

// Example
// Input:
// Source Networks       : Internal (group)
// OBJ-10.11.12.0_23 (10.11.12.0/23)
// 10.0.0.0/8
// Output:
// ("Source Networks", ["Internal (group)", "OBJ-10.11.12.0_23 (10.11.12.0/23)", "10.0.0.0/8"])
pub fn extract_name(lines: &[String]) -> Result<(String, Vec<String>), UtilitiesError> {
    if lines.is_empty() {
        return Err(UtilitiesError::NameExtractionError(
            "Input lines are empty".to_string(),
        ));
    }

    let first_line: Vec<_> = lines[0].split(": ").collect();
    if first_line.len() != 2 {
        return Err(UtilitiesError::NameExtractionError2(
            lines[0].to_string(),
            "Incorrect object first line format, expected '____ _____ : group or prefix'"
                .to_string(),
        ));
    }
    let name = first_line
        .first()
        .ok_or_else(|| {
            UtilitiesError::NameExtractionError2(
                lines[0].to_string(),
                "Missing object name in first line".to_string(),
            )
        })?
        .trim()
        .to_string();
    let merged_lines: Vec<_> = first_line[1..]
        .iter()
        .map(|x| x.to_string())
        .chain(lines[1..].iter().map(|x| x.to_string()))
        .collect();

    Ok((name, merged_lines))
}

// Example1:
// Internal (group)
//   OBJ-157.121.0.0 (157.121.0.0/16)
//   OBJ-206.213.0.0 (206.213.0.0/16)
//   OBJ-167.69.0.0 (167.69.0.0/16)
//   OBJ-198.187.64.0_18 (198.187.64.0/18)
//   10.0.0.0/8
//   204.99.0.0/16
//   172.16.0.0/12
// OBJ-192.168.243.0_24 (192.168.243.0/24)
// OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)
// return 8

// Example2:
// Internal (group)
// Another (group)
// return 1
pub fn calculate_lines_in_group(lines: &[String]) -> Result<usize, UtilitiesError> {
    if lines.is_empty() {
        return Err(UtilitiesError::GroupLineCalculationError(
            "Input lines are empty".to_string(),
        ));
    }
    if lines.len() == 1 {
        return Ok(1);
    }

    let [_, first_line, ..] = lines else {
        return Err(UtilitiesError::GroupLineCalculationError(format!(
            "Panic {:?}",
            lines
        )));
    };

    let reference_padding = first_line.len() - first_line.trim_start().len();
    let mut idx = 1;
    while idx < lines.len() {
        if lines[idx].contains("(group)") {
            return Ok(idx);
        }
        let padding = lines[idx].len() - lines[idx].trim_start().len();
        if padding != reference_padding {
            return Ok(idx);
        }
        idx += 1;
    }
    Ok(idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name_valid() {
        let lines = vec![
            "Source Networks       : Internal (group)".to_string(),
            "OBJ-10.11.12.0_23 (10.11.12.0/23)".to_string(),
            "10.0.0.0/8".to_string(),
        ];
        let (name, merged_lines) = extract_name(&lines).unwrap();
        assert_eq!(name, "Source Networks");
        assert_eq!(
            merged_lines,
            vec![
                "Internal (group)".to_string(),
                "OBJ-10.11.12.0_23 (10.11.12.0/23)".to_string(),
                "10.0.0.0/8".to_string(),
            ]
        );
    }

    #[test]
    fn test_extract_name_invalid_format() {
        let lines = vec!["Source Networks Internal (group)".to_string()];
        let result = extract_name(&lines);
        assert!(result.is_err());
        if let Err(UtilitiesError::NameExtractionError2(line, msg)) = result {
            assert_eq!(line, "Source Networks Internal (group)");
            assert_eq!(
                msg,
                "Incorrect object first line format, expected '____ _____ : group or prefix'"
            );
        } else {
            panic!("Expected UtilitiesError::NameExtractionError2");
        }
    }

    #[test]
    fn test_extract_name_empty_lines() {
        let lines: Vec<String> = vec![];
        let result = extract_name(&lines);
        assert!(result.is_err());
        if let Err(UtilitiesError::NameExtractionError(msg)) = result {
            assert_eq!(msg, "Input lines are empty");
        } else {
            panic!("Expected UtilitiesError::NameExtractionError");
        }
    }

    #[test]
    fn test_calculate_lines_in_group_single_group() {
        let lines = vec![
            "Internal (group)".to_string(),
            "  OBJ-157.121.0.0 (157.121.0.0/16)".to_string(),
            "  OBJ-206.213.0.0 (206.213.0.0/16)".to_string(),
            "  OBJ-167.69.0.0 (167.69.0.0/16)".to_string(),
            "  OBJ-198.187.64.0_18 (198.187.64.0/18)".to_string(),
            "  10.0.0.0/8".to_string(),
            "  204.99.0.0/16".to_string(),
            "  172.16.0.0/12".to_string(),
            "OBJ-192.168.243.0_24 (192.168.243.0/24)".to_string(),
            "OBJ-10.18.46.62-69 (10.18.46.62-10.18.46.69)".to_string(),
        ];
        let result = calculate_lines_in_group(&lines).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_calculate_lines_in_group_multiple_groups() {
        let lines = vec![
            "Internal (group)".to_string(),
            "Another (group)".to_string(),
        ];
        let result = calculate_lines_in_group(&lines).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_calculate_lines_in_group_empty_lines() {
        let lines: Vec<String> = vec![];
        let result = calculate_lines_in_group(&lines);
        assert!(result.is_err());
        if let Err(UtilitiesError::GroupLineCalculationError(msg)) = result {
            assert_eq!(msg, "Input lines are empty");
        } else {
            panic!("Expected UtilitiesError::GroupLineCalculationError");
        }
    }
}
