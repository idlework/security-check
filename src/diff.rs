use crate::check::{CheckResult, Status};
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn state_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".security-check")
}

fn state_file() -> PathBuf {
    state_dir().join("last.json")
}

pub fn save_results(results: &[CheckResult]) {
    let dir = state_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let json = serde_json::to_string(results).unwrap_or_default();
    let _ = fs::write(state_file(), json);
}

pub fn load_previous() -> Option<Vec<CheckResult>> {
    let data = fs::read_to_string(state_file()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn print_diff(current: &[CheckResult], previous: &[CheckResult]) {
    let prev_map: HashMap<&str, &Status> = previous
        .iter()
        .map(|r| (r.name.as_str(), &r.status))
        .collect();

    let mut changes: Vec<String> = Vec::new();
    let mut new_checks: Vec<&str> = Vec::new();

    for check in current {
        match prev_map.get(check.name.as_str()) {
            Some(prev_status) if **prev_status != check.status => {
                let arrow = match (&check.status, prev_status) {
                    (Status::Pass, _) => format!(
                        "  {} {} → {}",
                        check.name,
                        format!("{}", prev_status).red(),
                        format!("{}", check.status).green(),
                    ),
                    (Status::Fail, _) => format!(
                        "  {} {} → {}",
                        check.name,
                        format!("{}", prev_status).green(),
                        format!("{}", check.status).red(),
                    ),
                    _ => format!(
                        "  {} {} → {}",
                        check.name,
                        prev_status,
                        check.status,
                    ),
                };
                changes.push(arrow);
            }
            None if check.status != Status::Skip => {
                new_checks.push(&check.name);
            }
            _ => {}
        }
    }

    if changes.is_empty() && new_checks.is_empty() {
        println!("  {}", "No changes since last run.".dimmed());
        println!();
        return;
    }

    println!("  {}", "Changes since last run:".white().bold());
    println!();
    for change in &changes {
        println!("{}", change);
    }
    if !new_checks.is_empty() {
        for name in &new_checks {
            println!("  {} {}", "NEW".cyan().bold(), name);
        }
    }
    println!();
}
