use crate::check::{count_by_status, Category, CheckResult, Status};
use crate::scoring::Score;
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
        let checks: Vec<&CheckResult> =
            results.iter().filter(|r| r.category == *category).collect();
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

            println!(
                "  {}  {:<34}  {}",
                colored_status, check.name, check.message
            );

            if verbose {
                if let Some(detail) = &check.detail {
                    println!("  {}  {}", "      ", detail.dimmed());
                }
            }

            if let (Status::Warn | Status::Fail, Some(hint)) =
                (check.status, &check.fix_hint)
            {
                println!("  {}  {}", "      ", format!("Hint: {}", hint).dimmed());
            }
        }
        println!();
    }
}

pub fn print_summary(results: &[CheckResult], is_root: bool) {
    let (passed, warned, failed, skipped) = count_by_status(results);

    let score = Score::from_results(results);
    let pct = score.percentage();

    println!("  {}", "Summary".white().bold());
    println!();
    println!(
        "  {} passed  {} warnings  {} failures  {} skipped",
        passed.to_string().green().bold(),
        warned.to_string().yellow().bold(),
        failed.to_string().red().bold(),
        skipped.to_string().dimmed(),
    );
    println!();

    let score_line = format!(
        "  Score: {}% ({}) -- {}/{} points",
        pct,
        score.grade(),
        score.earned,
        score.possible
    );
    let colored_score = match pct {
        90..=100 => score_line.green().bold(),
        70..=89 => score_line.yellow().bold(),
        _ => score_line.red().bold(),
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
