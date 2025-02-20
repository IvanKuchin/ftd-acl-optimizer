pub struct Reader {
    lines: Vec<String>,
}

impl From <Vec<String>> for Reader {
    fn from(lines: Vec<String>) -> Self {
        Self {
            lines
        }
    }
}

impl Reader {
    pub fn get_until_matching_line(&mut self, pattern: &str) -> Option<String> {
        let extra: Vec<_> = self.lines.iter()
            .take_while(|line| !line.contains(pattern))
            .map(|s| 1)
            .collect();
        self.lines.drain(0..extra.len());

        let line: Vec<_> = self.lines.iter()
            .take(1)
            .map(|s| s.to_string())
            .collect();
        self.lines.drain(0..line.len());

        if !line.is_empty() {
            Some(line.get(0).unwrap().to_string())
        } else {
            None
        }

    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_until_matching_line_found() {
        let lines = vec![
            "first line".to_string(),
            "second line".to_string(),
            "matching line".to_string(),
            "fourth line".to_string(),
        ];
        let mut reader = Reader::from(lines);
        let result = reader.get_until_matching_line("matching");
        assert_eq!(result, Some("matching line".to_string()));
        assert_eq!(reader.lines, vec!["fourth line".to_string()]);
    }

    #[test]
    fn test_get_until_matching_line_not_found() {
        let lines = vec![
            "first line".to_string(),
            "second line".to_string(),
            "third line".to_string(),
        ];
        let mut reader = Reader::from(lines);
        let result = reader.get_until_matching_line("matching");
        assert_eq!(result, None);
        assert_eq!(reader.lines, Vec::<String>::new());
    }

    #[test]
    fn test_get_until_matching_line_empty() {
        let lines: Vec<String> = vec![];
        let mut reader = Reader::from(lines);
        let result = reader.get_until_matching_line("matching");
        assert_eq!(result, None);
        assert_eq!(reader.lines, Vec::<String>::new());
    }

    #[test]
    fn test_get_until_matching_line_first_line() {
        let lines = vec![
            "matching line".to_string(),
            "second line".to_string(),
            "third line".to_string(),
        ];
        let mut reader = Reader::from(lines);
        let result = reader.get_until_matching_line("matching");
        assert_eq!(result, Some("matching line".to_string()));
        assert_eq!(reader.lines, vec![
            "second line".to_string(),
            "third line".to_string(),
        ]);
    }

    #[test]
    fn test_get_until_matching_line_last_line() {
        let lines = vec![
            "first line".to_string(),
            "second line".to_string(),
            "matching line".to_string(),
        ];
        let mut reader = Reader::from(lines);
        let result = reader.get_until_matching_line("matching");
        assert_eq!(result, Some("matching line".to_string()));
        assert_eq!(reader.lines, Vec::<String>::new());
    }
    
    #[test]
    fn test_get_until_matching_line_multiple_matches() {
        let lines = vec![
            "first line".to_string(),
            "matching line".to_string(),
            "second matching line".to_string(),
            "third line".to_string(),
        ];
        let mut reader = Reader::from(lines);
        let result = reader.get_until_matching_line("matching");
        assert_eq!(result, Some("matching line".to_string()));
        assert_eq!(reader.lines, vec!["second matching line".to_string(), "third line".to_string()]);
    }
}
