/// Reads the next rule from the reader's lines.
///
/// This method searches for the next rule in the reader's lines, starting with a line that contains "Rule: "
/// and including all subsequent lines until the next line that contains "Rule: " or the end of the lines.
///
/// # Returns
///
/// An `Option` containing a `Vec<String>` with the lines of the next rule if found, or `None` if no more rules are present.
///
/// # Examples
///
/// ```rust
/// let mut reader = Reader { lines: vec![
///     "Some text".to_string(),
///     "Rule: First Rule".to_string(),
///     "First rule body line 1".to_string(),
///     "First rule body line 2".to_string(),
///     "Rule: Second Rule".to_string(),
///     "Second rule body line 1".to_string(),
/// ]};
///
/// assert_eq!(reader.next_rule(), Some(vec![
///     "Rule: First Rule".to_string(),
///     "First rule body line 1".to_string(),
///     "First rule body line 2".to_string(),
/// ]));
///
/// assert_eq!(reader.next_rule(), Some(vec![
///     "Rule: Second Rule".to_string(),
///     "Second rule body line 1".to_string(),
/// ]));
///
/// assert_eq!(reader.next_rule(), None);
/// ```

pub struct Reader {
    lines: Vec<String>,
}

impl Reader {
    pub fn next_rule(&mut self) -> Option<Vec<String>> {
  
        let extra: Vec<_> = self.lines.iter()
            .take_while(|line| !line.contains("Rule: "))
            .map(|s| 1)
            .collect();
        self.lines.drain(0..extra.len());
        
        let rule_title: Vec<_> = self.lines.iter()
            .skip_while(|line| !line.contains("Rule: "))
            .take(1)
            .map(|s| s.to_string())
            .collect();
        self.lines.drain(0..rule_title.len());
        
        // dbg!(&rule_title);
        // dbg!(&self.lines);
        
        let rule_body: Vec<_> = self.lines.iter()
            .take_while(|line| !line.contains("Rule: "))
            .map(|s| s.to_string())
            .collect();
        self.lines.drain(0..rule_body.len());

        // dbg!(&rule_body);
        // dbg!(&self.lines);

        let rule_lines: Vec<_> = rule_title.iter()
            .chain(rule_body.iter())
            .map(|s| s.to_string())
            .collect();

        if !rule_lines.is_empty() {
            Some(rule_lines)
        } else {
            None
        }
    }
}

impl From<Vec<String>> for Reader {
    fn from(lines: Vec<String>) -> Self {
        Self {
            lines
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_rule_single_rule() {
        let mut reader = Reader { lines: vec![
            "Rule: Only Rule".to_string(),
            "Only rule body line 1".to_string(),
            "Only rule body line 2".to_string(),
        ]};

        assert_eq!(reader.next_rule(), Some(vec![
            "Rule: Only Rule".to_string(),
            "Only rule body line 1".to_string(),
            "Only rule body line 2".to_string(),
        ]));

        assert_eq!(reader.next_rule(), None);
    }

    #[test]
    fn test_next_rule_no_rules() {
        let mut reader = Reader { lines: vec![
            "Some text".to_string(),
            "Some more text".to_string(),
        ]};

        assert_eq!(reader.next_rule(), None);
    }

    #[test]
    fn test_next_rule_empty_lines() {
        let mut reader = Reader { lines: vec![] };

        assert_eq!(reader.next_rule(), None);
    }

    #[test]
    fn test_next_rule_multiple_rules_with_empty_lines() {
        let mut reader = Reader { lines: vec![
            "".to_string(),
            "Rule: First Rule".to_string(),
            "First rule body line 1".to_string(),
            "".to_string(),
            "First rule body line 2".to_string(),
            "".to_string(),
            "Rule: Second Rule".to_string(),
            "".to_string(),
            "Second rule body line 1".to_string(),
            "".to_string(),
        ]};

        assert_eq!(reader.next_rule(), Some(vec![
            "Rule: First Rule".to_string(),
            "First rule body line 1".to_string(),
            "".to_string(),
            "First rule body line 2".to_string(),
            "".to_string(),
        ]));

        assert_eq!(reader.next_rule(), Some(vec![
            "Rule: Second Rule".to_string(),
            "".to_string(),
            "Second rule body line 1".to_string(),
            "".to_string(),
        ]));

        assert_eq!(reader.next_rule(), None);
    }

    #[test]
    fn test_next_rule_with_intermediate_text() {
        let mut reader = Reader { lines: vec![
            "Some text".to_string(),
            "Rule: First Rule".to_string(),
            "First rule body line 1".to_string(),
            "First rule body line 2".to_string(),
            "Some intermediate text".to_string(),
            "Rule: Second Rule".to_string(),
            "Second rule body line 1".to_string(),
        ]};

        assert_eq!(reader.next_rule(), Some(vec![
            "Rule: First Rule".to_string(),
            "First rule body line 1".to_string(),
            "First rule body line 2".to_string(),
            "Some intermediate text".to_string(),
        ]));

        assert_eq!(reader.next_rule(), Some(vec![
            "Rule: Second Rule".to_string(),
            "Second rule body line 1".to_string(),
        ]));

        assert_eq!(reader.next_rule(), None);
    }
}
