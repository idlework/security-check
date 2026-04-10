mod check;
mod checks;
mod diff;
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

    /// Show changes since last run
    #[arg(long)]
    diff: bool,
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

fn all_check_batches(ctx: &Context, hw_output: &str) -> Vec<Vec<CheckResult>> {
    vec![
        checks::system::run_checks(hw_output),
        checks::encryption::run_checks(),
        checks::firewall::run_checks(),
        checks::malware::run_checks(),
        checks::updates::run_checks(),
        checks::network::run_checks(ctx),
        checks::hardware::run_checks(hw_output),
        checks::user_security::run_checks(),
        checks::privacy::run_checks(),
    ]
}

fn main() {
    let cli = Cli::parse();
    let ctx = Context::detect();
    let hw_output = runner::run_command("system_profiler", &["SPHardwareDataType"])
        .unwrap_or_default();

    if cli.list {
        println!("Available checks:\n");
        let results: Vec<CheckResult> = all_check_batches(&ctx, &hw_output)
            .into_iter()
            .flatten()
            .collect();
        for cat in Category::all() {
            let count = results.iter().filter(|r| r.category == *cat).count();
            println!("  {} ({} checks)", cat.label(), count);
        }
        return;
    }

    let filter = cli.category.as_ref().map(|f| {
        f.to_lowercase().replace([' ', '-'], "_")
    });

    let previous = if cli.diff { diff::load_previous() } else { None };

    // JSON mode: collect everything, then output
    if cli.json {
        let mut results: Vec<CheckResult> = all_check_batches(&ctx, &hw_output)
            .into_iter()
            .flatten()
            .collect();
        if let Some(ref f) = filter {
            results.retain(|r| r.category.matches_filter(f));
        }
        diff::save_results(&results);
        let report = JsonReport {
            system_info: gather_system_info(),
            summary: JsonSummary::from_results(&results),
            checks: results,
        };
        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
        return;
    }

    // Terminal mode: print each category as it completes
    output::print_header(&gather_system_info());

    let mut results = Vec::new();
    for batch in all_check_batches(&ctx, &hw_output) {
        let batch: Vec<CheckResult> = if let Some(ref f) = filter {
            batch.into_iter().filter(|r| r.category.matches_filter(f)).collect()
        } else {
            batch
        };
        output::print_category(&batch, cli.verbose);
        results.extend(batch);
    }

    output::print_summary(&results, ctx.is_root);

    if let Some(prev) = previous {
        diff::print_diff(&results, &prev);
    }

    diff::save_results(&results);
}
