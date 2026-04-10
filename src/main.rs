mod check;
mod checks;
mod output;
mod runner;
mod scoring;

use check::{count_by_status, Category, CheckResult};
use clap::Parser;
use runner::Context;
use scoring::Score;
use serde::Serialize;

#[derive(Parser)]
#[command(name = "security-check")]
#[command(about = "macOS security audit tool")]
#[command(version)]
struct Cli {
    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Show detailed information for each check
    #[arg(short, long)]
    verbose: bool,

    /// Run only checks in a specific category
    #[arg(long)]
    category: Option<String>,

    /// List all available checks without running them
    #[arg(long)]
    list: bool,
}

#[derive(Serialize)]
struct JsonReport {
    system_info: String,
    checks: Vec<CheckResult>,
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonSummary {
    passed: usize,
    warnings: usize,
    failures: usize,
    skipped: usize,
    score_pct: u32,
    grade: String,
}

impl JsonSummary {
    fn from_results(results: &[CheckResult]) -> Self {
        let (passed, warnings, failures, skipped) = count_by_status(results);
        let score = Score::from_results(results);
        Self {
            passed,
            warnings,
            failures,
            skipped,
            score_pct: score.percentage(),
            grade: score.grade().to_string(),
        }
    }
}

fn gather_system_info() -> String {
    let model = runner::run_command("sysctl", &["-n", "hw.model"])
        .unwrap_or_default()
        .trim()
        .to_string();

    let chip = runner::run_command("sysctl", &["-n", "machdep.cpu.brand_string"])
        .unwrap_or_default()
        .trim()
        .to_string();

    let os_version = runner::run_command("sw_vers", &["-productVersion"])
        .unwrap_or_default()
        .trim()
        .to_string();

    let os_name = runner::run_command("sw_vers", &["-productName"])
        .unwrap_or_default()
        .trim()
        .to_string();

    if model.is_empty() {
        return String::new();
    }

    format!("{} ({}) -- {} {}", model, chip, os_name, os_version)
}

fn run_all_checks(ctx: &Context) -> Vec<CheckResult> {
    let mut results = Vec::new();
    results.extend(checks::system::run_checks());
    results.extend(checks::encryption::run_checks());
    results.extend(checks::firewall::run_checks());
    results.extend(checks::malware::run_checks());
    results.extend(checks::updates::run_checks());
    results.extend(checks::network::run_checks(ctx));
    results.extend(checks::hardware::run_checks());
    results.extend(checks::user_security::run_checks());
    results.extend(checks::privacy::run_checks());
    results
}

fn main() {
    let cli = Cli::parse();
    let ctx = Context::detect();

    if cli.list {
        println!("Available checks:\n");
        let results = run_all_checks(&ctx);
        for cat in Category::all() {
            let count = results.iter().filter(|r| r.category == *cat).count();
            println!("  {} ({} checks)", cat.label(), count);
        }
        return;
    }

    let mut results = run_all_checks(&ctx);

    if let Some(ref filter) = cli.category {
        let filter = filter.to_lowercase().replace([' ', '-'], "_");
        results.retain(|r| r.category.matches_filter(&filter));
    }

    if cli.json {
        let report = JsonReport {
            system_info: gather_system_info(),
            summary: JsonSummary::from_results(&results),
            checks: results,
        };
        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
        return;
    }

    output::print_header(&gather_system_info());
    output::print_results(&results, cli.verbose);
    output::print_summary(&results, ctx.is_root);
}
