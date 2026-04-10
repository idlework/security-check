mod check;
mod checks;
mod output;
mod runner;
mod scoring;

use check::{Category, CheckResult, Status};
use clap::Parser;
use runner::Context;
use serde::Serialize;

#[derive(Parser)]
#[command(name = "security-cli")]
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
        for cat in Category::all() {
            let dummy_results = run_all_checks(&ctx);
            let count = dummy_results.iter().filter(|r| r.category == *cat).count();
            println!("  {} ({} checks)", cat.label(), count);
        }
        return;
    }

    let mut results = run_all_checks(&ctx);

    if let Some(ref category_filter) = cli.category {
        let filter_lower = category_filter.to_lowercase().replace([' ', '-'], "_");
        results.retain(|r| {
            let cat_str = format!("{:?}", r.category).to_lowercase();
            cat_str.contains(&filter_lower)
        });
    }

    if cli.json {
        let system_info = gather_system_info();
        let (score, possible) = scoring::calculate_score(&results);
        let pct = if possible > 0 {
            (score as f64 / possible as f64 * 100.0).round() as u32
        } else {
            0
        };

        let report = JsonReport {
            system_info,
            summary: JsonSummary {
                passed: results.iter().filter(|r| r.status == Status::Pass).count(),
                warnings: results.iter().filter(|r| r.status == Status::Warn).count(),
                failures: results.iter().filter(|r| r.status == Status::Fail).count(),
                skipped: results.iter().filter(|r| r.status == Status::Skip).count(),
                score_pct: pct,
                grade: scoring::grade(pct).to_string(),
            },
            checks: results,
        };

        let json = serde_json::to_string_pretty(&report).unwrap_or_default();
        println!("{}", json);
        return;
    }

    let system_info = gather_system_info();
    output::print_header(&system_info);
    output::print_results(&results, cli.verbose);
    output::print_summary(&results, ctx.is_root);
}
