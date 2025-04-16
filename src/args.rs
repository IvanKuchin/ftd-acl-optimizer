use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version, about, author)]
pub struct AppArgs {
    /// Output of "show access-control-config"
    #[arg(short, long, required = true)]
    pub file: PathBuf,

    #[clap(subcommand)]
    /// Command to run
    pub subcommand: Verb,
}

#[derive(Subcommand, Debug)]
pub enum Verb {
    #[clap(subcommand)]
    /// Analyze a rule or whole access policy from "show access-control-config"
    Get(Entity),
}

#[derive(Subcommand, Debug)]
pub enum Entity {
    #[clap(subcommand)]
    /// Analyze a rule from "show access-control-config"
    Rule(Rule),

    #[clap(subcommand)]
    /// Get info about top-k rules from "show access-control-config"
    TopK(TopK),

    #[clap(subcommand)]
    /// Analyze the whole access policy from "show access-control-config"
    Acp(Acp),
}

#[derive(Subcommand, Debug)]
/// Analyze a rule from "show access-control-config"
pub enum Rule {
    /// Analyze a rule capacity and optimization capacity
    Capacity(RuleName),

    /// Get optimization report for a rule
    Analysis(RuleName),
}

#[derive(Args, Debug)]
/// Rule name from "show access-control-config"
pub struct RuleName {
    /// Rule name to analyze
    pub name: String,
}

#[derive(Subcommand, Debug)]
/// Get info about top-k rules from "show access-control-config"
pub enum TopK {
    /// Get top-k rules by capacity
    ByCapacity(TopKByCapacity),

    /// Get top-k rules by optimization (ratio of a current capacity to an optimized capacity)
    ByOptimization(TopKByOptimization),
}

#[derive(Args, Debug)]
/// Get top-k rules by capacity
pub struct TopKByCapacity {}

#[derive(Args, Debug)]
/// Get top-k rules by optimization (ratio of a current capacity to an optimized capacity)
pub struct TopKByOptimization {}

#[derive(Subcommand, Debug)]
/// Analyze the whole access policy from "show access-control-config"
pub enum Acp {
    /// Analyze and produce report of the whole access policy from "show access-control-config"
    Analysis(AcpAnalysis),

    /// Get capacity optimization only for each rule in the access policy
    Capacity(AcpCapacity),
}

#[derive(Args, Debug)]
pub struct AcpAnalysis {}

#[derive(Args, Debug)]
pub struct AcpCapacity {}
