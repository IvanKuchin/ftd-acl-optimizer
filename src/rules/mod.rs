use std::ops::Deref;

mod reader;
use reader::Reader;

pub mod rule;
use rule::Rule;

#[derive(thiserror::Error, Debug)]
pub enum RulesError {
    #[error("Fail to parse rules: {0}")]
    General(String),
    #[error("Failed to parse rule: {0}")]
    ParseRule(#[from] rule::RuleError),
}

#[derive(Debug)]
pub struct Rules(Vec<Rule>);

impl Deref for Rules {
    type Target = Vec<Rule>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Vec<String>> for Rules {
    type Error = RulesError;

    fn try_from(lines: Vec<String>) -> Result<Self, Self::Error> {
        let mut reader = Reader::from(lines);

        let mut rules = vec![];

        while let Some(rule_lines) = reader.next_rule() {
            // dbg!(&rule_lines);
            let rule = Rule::try_from(rule_lines)?;
            rules.push(rule);
        }

        Ok(Self(rules))
    }
}
