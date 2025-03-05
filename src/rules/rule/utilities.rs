#[derive(thiserror::Error, Debug)]
pub enum UtilitiesError {
    #[error("Fail to extract name: {0}")]
    NameExtractionError(String),
    #[error("Fail to extract name with details: {0}, {1}")]
    NameExtractionError2(String, String),
}

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
            "Incorrect first line format, expected '____ _____ : group or prefix'".to_string(),
        ));
    }
    let name = first_line
        .first()
        .ok_or_else(|| {
            UtilitiesError::NameExtractionError2(
                lines[0].to_string(),
                "Missing name in first line".to_string(),
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
                "Incorrect first line format, expected '____ _____ : group or prefix'"
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
}
