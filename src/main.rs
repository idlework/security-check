mod check;
mod checks;
mod output;
mod runner;
mod scoring;

use check::CheckResult;
use clap::Parser;
use runner::Context;

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

fn run_all_checks(_ctx: &Context) -> Vec<CheckResult> {
    let mut results = Vec::new();
    results.extend(checks::system::run_checks());
    results.extend(checks::encryption::run_checks());
    results.extend(checks::firewall::run_checks());
    results.extend(checks::malware::run_checks());
    results.extend(checks::updates::run_checks());
    results
}

fn main() {
    let cli = Cli::parse();
    let ctx = Context::detect();

    if cli.list {
        println!("Available check categories:");
        for cat in check::Category::all() {
            println!("  - {}", cat.label());
        }
        return;
    }

    let mut results = run_all_checks(&ctx);

    if let Some(ref category_filter) = cli.category {
        let filter_lower = category_filter.to_lowercase().replace(' ', "_");
        results.retain(|r| {
            let cat_str = format!("{:?}", r.category).to_lowercase();
            cat_str.contains(&filter_lower)
        });
    }

    if cli.json {
        let json = serde_json::to_string_pretty(&results).unwrap_or_default();
        println!("{}", json);
        return;
    }

    let system_info = gather_system_info();
    output::print_header(&system_info);
    output::print_results(&results, cli.verbose);
    output::print_summary(&results, ctx.is_root);
}
