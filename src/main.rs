use core::panic;
use rules::Rules;
use std::path::PathBuf;

pub mod rules;

use clap::{arg, command, value_parser, Command};

#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Fail to parse rule: {0}")]
    Rule(#[from] rules::rule::RuleError),
    #[error("Fail to parse rules: {0}")]
    Rules(#[from] rules::RulesError),
    #[error("No rule found with name: {name}")]
    RuleEmpty { name: String },
    #[error("No rules found")]
    RulesEmpty(),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

fn main() {
    let matches = command!() // requires `cargo` feature
        .subcommand_required(true)
        .arg(arg!(
            -d --debug ... "Turn debugging information on"
        ))
        .subcommand(
            Command::new("analyze")
                .arg_required_else_help(true)
                .about("Analyze rule from 'show access-control-config'")
                .arg(
                    arg!(
                        -f --file <FILE> "(required) Output of show access-control-config"
                    )
                    // We don't have syntax yet for optional options, so manually calling `required`
                    .required(true)
                    .value_parser(value_parser!(PathBuf)),
                )
                .arg(arg!(
                    -r --rule <RULE> "(optional) Rule name to analyze"
                )),
        )
        .get_matches();

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    let _trace_level = {
        match matches.get_one::<u8>("debug") {
            None => panic!("Debug flag was not set"),
            Some(0) => 0,
            Some(1) => 1,
            Some(2) => 2,
            Some(_) => panic!("Debug flag was set more than twice"),
        }
    };

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    if let Some(matches) = matches.subcommand_matches("analyze") {
        let file_path = matches.get_one::<PathBuf>("file").expect("file not found");
        let rule = matches.get_one::<String>("rule");

        if let Some(rule) = rule {
            match analyze_rule(file_path, rule) {
                Ok(()) => {}
                Err(e) => println!("Analysis failed: {}", e),
            }
        } else {
            todo!();
        }
    }
}

fn is_filtered(line: &str) -> bool {
    line.contains("Object missing: ") || line.contains("")
}

fn analyze_rule(fname: &PathBuf, rule: &String) -> Result<(), MyError> {
    let rule_lines: Vec<_> = std::fs::read_to_string(fname)?
        .lines()
        .skip_while(|line| !line.contains(&format!("Rule: {}", rule)))
        .take_while(|line| !line.contains("Rule: ") || line.contains(&format!("Rule: {}", rule)))
        .filter(|line| !is_filtered(line))
        .map(|s| s.to_string())
        .collect();

    let rules = Rules::try_from(rule_lines)?;

    if rules.is_empty() {
        return Err(MyError::RuleEmpty { name: rule.clone() });
    }

    // dbg!(&rules);
    println!("# of rules found: {}", rules.len());
    println!("rule capacity: {}", rules.capacity());
    todo!("Implement rule analysis");
}

fn _analyze_rules(fname: &PathBuf) -> Result<(), MyError> {
    let rule_lines: Vec<_> = std::fs::read_to_string(fname)?
        .lines()
        .skip_while(|line| !line.contains("Rule: "))
        .filter(|line| is_filtered(line))
        .map(|s| s.to_string())
        .collect();

    let rules = Rules::try_from(rule_lines)?;

    if rules.is_empty() {
        return Err(MyError::RulesEmpty {});
    }

    todo!("Implement rules analysis");
}
