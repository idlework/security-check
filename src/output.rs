use crate::check::{Category, CheckResult, Status};
use crate::scoring::{calculate_score, grade};
use colored::Colorize;

pub fn print_header(system_info: &str) {
    println!();
    println!("{}", "  macOS Security Audit".cyan().bold());
    if !system_info.is_empty() {
        println!("  {}", system_info.dimmed());
    }
    println!();
}

pub fn print_results(results: &[CheckResult], verbose: bool) {
    for category in Category::all() {
        let checks: Vec<&CheckResult> = results.iter().filter(|r| r.category == *category).collect();
        if checks.is_empty() {
            continue;
        }

        println!("  {}", category.label().white().bold());
        println!();

        for check in &checks {
            let status_str = format!(" {:4} ", check.status);
            let colored_status = match check.status {
                Status::Pass => status_str.green().bold(),
                Status::Warn => status_str.yellow().bold(),
                Status::Fail => status_str.red().bold(),
                Status::Skip => status_str.dimmed(),
            };

            let name = format!("{:<34}", check.name);
            println!("  {}  {}  {}", colored_status, name, check.message);

            if verbose {
                if let Some(detail) = &check.detail {
                    println!("  {}  {}", "      ", detail.dimmed());
                }
            }

            if matches!(check.status, Status::Warn | Status::Fail) {
                if let Some(hint) = &check.fix_hint {
                    println!(
                        "  {}  {}",
                        "      ",
                        format!("Hint: {}", hint).dimmed()
                    );
                }
            }
        }
        println!();
    }
}

pub fn print_summary(results: &[CheckResult], is_root: bool) {
    let passed = results.iter().filter(|r| r.status == Status::Pass).count();
    let warned = results.iter().filter(|r| r.status == Status::Warn).count();
    let failed = results.iter().filter(|r| r.status == Status::Fail).count();
    let skipped = results.iter().filter(|r| r.status == Status::Skip).count();

    let (score, possible) = calculate_score(results);
    let pct = if possible > 0 {
        (score as f64 / possible as f64 * 100.0).round() as u32
    } else {
        0
    };
    let letter = grade(pct);

    println!("  {}", "Summary".white().bold());
    println!();

    let summary = format!(
        "  {} passed  {} warnings  {} failures  {} skipped",
        passed.to_string().green().bold(),
        warned.to_string().yellow().bold(),
        failed.to_string().red().bold(),
        skipped.to_string().dimmed(),
    );
    println!("{}", summary);
    println!();

    let score_line = format!("  Score: {}% ({}) -- {}/{} points", pct, letter, score, possible);
    let colored_score = if pct >= 90 {
        score_line.green().bold()
    } else if pct >= 70 {
        score_line.yellow().bold()
    } else {
        score_line.red().bold()
    };
    println!("{}", colored_score);

    if !is_root && skipped > 0 {
        println!();
        println!(
            "  {}",
            "Run with sudo for complete results: sudo security-check".dimmed()
        );
    }

    println!();
}
