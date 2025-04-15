use std::ops::Deref;

mod reader;
use reader::Reader;

pub mod rule;
use rule::Rule;
use std::convert::TryFrom;

#[derive(thiserror::Error, Debug)]
pub enum AcpError {
    #[error("Fail to parse rules: {0}")]
    General(String),
    #[error("Failed to parse rule: {0}")]
    ParseRule(#[from] rule::RuleError),
}

#[derive(Debug)]
pub struct Acp(Vec<Rule>);

impl Deref for Acp {
    type Target = Vec<Rule>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Vec<String>> for Acp {
    type Error = AcpError;

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

impl Acp {
    pub fn capacity(&self) -> u64 {
        self.iter().map(|r| r.capacity()).sum()
    }

    pub fn optimized_capacity(&self) -> u64 {
        self.iter().map(|r| r.optimized_capacity()).sum()
    }

    pub fn rule_count(&self) -> usize {
        self.len()
    }

    pub fn rule_by_name(&self, rule_name: &str) -> Option<&Rule> {
        self.iter().find(|r| r.get_name() == rule_name)
    }

    pub fn rule_by_idx(&self, idx: usize) -> Option<&Rule> {
        self.get(idx)
    }
}
