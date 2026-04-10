use crate::check::{Category, CheckResult};
use crate::runner::run_command;
use std::time::SystemTime;

pub fn run_checks() -> Vec<CheckResult> {
    let hw_output = run_command("system_profiler", &["SPHardwareDataType"]).unwrap_or_default();
    vec![
        check_system_info(&hw_output),
        check_activation_lock(&hw_output),
        check_time_machine(),
        check_backup_recency(),
    ]
}

fn extract_field<'a>(output: &'a str, field: &str) -> Option<&'a str> {
    output
        .lines()
        .find(|l| l.contains(field))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim())
}

fn check_system_info(hw_output: &str) -> CheckResult {
    let model = extract_field(hw_output, "Model Name").unwrap_or("Unknown");
    let chip = extract_field(hw_output, "Chip")
        .or_else(|| extract_field(hw_output, "Processor Name"))
        .unwrap_or("Unknown");
    let memory = extract_field(hw_output, "Memory").unwrap_or("Unknown");
    let serial: String = extract_field(hw_output, "Serial Number")
        .map(|s| {
            let chars: Vec<char> = s.chars().collect();
            if chars.len() > 4 {
                format!("...{}", chars[chars.len() - 4..].iter().collect::<String>())
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| "Unknown".into());

    CheckResult::pass(
        Category::Hardware,
        "System Info",
        &format!("{}, {}, {}, Serial {}", model, chip, memory, serial),
    )
    .with_weight(0) // informational only
}

fn check_activation_lock(hw_output: &str) -> CheckResult {
    match extract_field(hw_output, "Activation Lock Status") {
        Some(s) if s.contains("Enabled") => {
            CheckResult::pass(Category::Hardware, "Activation Lock", "Activation Lock is enabled")
                .with_weight(5)
        }
        Some(_) => {
            CheckResult::warn(Category::Hardware, "Activation Lock", "Activation Lock is disabled")
                .with_weight(5)
                .with_fix_hint("Enable Find My Mac in System Settings > Apple ID > iCloud > Find My Mac")
        }
        None => CheckResult::skip(
            Category::Hardware,
            "Activation Lock",
            "Could not determine Activation Lock status",
        ),
    }
}

fn check_time_machine() -> CheckResult {
    match run_command("tmutil", &["destinationinfo"]) {
        Ok(output) if output.contains("No destinations configured") => {
            CheckResult::warn(
                Category::Hardware,
                "Time Machine",
                "No backup destination configured",
            )
            .with_weight(5)
            .with_fix_hint("Enable Time Machine in System Settings > General > Time Machine")
        }
        Ok(output) if output.contains("Name") => {
            let dest = output
                .lines()
                .find(|l| l.contains("Name"))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim())
                .unwrap_or("configured");
            CheckResult::pass(
                Category::Hardware,
                "Time Machine",
                &format!("Backup destination: {}", dest),
            )
            .with_weight(5)
        }
        _ => CheckResult::skip(
            Category::Hardware,
            "Time Machine",
            "Could not determine Time Machine status",
        ),
    }
}

fn check_backup_recency() -> CheckResult {
    match run_command("tmutil", &["latestbackup"]) {
        Ok(output) => {
            let path = output.trim();
            // Try to get modification time of the backup path
            match std::fs::metadata(path).and_then(|m| m.modified()) {
                Ok(modified) => {
                    let days = SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or_default()
                        .as_secs()
                        / 86400;
                    match days {
                        0..=7 => CheckResult::pass(
                            Category::Hardware,
                            "Backup Recency",
                            &format!("Last backup: {} day(s) ago", days),
                        )
                        .with_weight(5),
                        8..=30 => CheckResult::warn(
                            Category::Hardware,
                            "Backup Recency",
                            &format!("Last backup: {} days ago", days),
                        )
                        .with_weight(5)
                        .with_fix_hint("Run a Time Machine backup soon"),
                        _ => CheckResult::warn(
                            Category::Hardware,
                            "Backup Recency",
                            &format!("Last backup: {} days ago (stale)", days),
                        )
                        .with_weight(5)
                        .with_fix_hint("Run a Time Machine backup immediately"),
                    }
                }
                Err(_) => {
                    // Can't stat the path, but tmutil returned something — extract date from path
                    // Paths look like: /Volumes/.../2025-03-15-123456
                    let basename = path.rsplit('/').next().unwrap_or("");
                    CheckResult::pass(
                        Category::Hardware,
                        "Backup Recency",
                        &format!("Last backup: {}", basename),
                    )
                    .with_weight(5)
                }
            }
        }
        Err(_) => CheckResult::skip(
            Category::Hardware,
            "Backup Recency",
            "No backups found or Time Machine not configured",
        ),
    }
}
