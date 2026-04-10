use crate::check::{Category, CheckResult};
use crate::runner::{run_command, run_defaults_read};
use std::fs;
use std::path::Path;

pub fn run_checks() -> Vec<CheckResult> {
    vec![
        check_airdrop(),
        check_user_launch_agents(),
        check_system_launch_daemons(),
        check_analytics_sharing(),
    ]
}

fn check_airdrop() -> CheckResult {
    match run_defaults_read("com.apple.sharingd", "DiscoverableMode") {
        Ok(output) => {
            let mode = output.trim();
            match mode {
                "Off" => CheckResult::pass(Category::Privacy, "AirDrop", "AirDrop is off")
                    .with_weight(3),
                "Contacts Only" | "ContactsOnly" => {
                    CheckResult::pass(Category::Privacy, "AirDrop", "AirDrop set to Contacts Only")
                        .with_weight(3)
                }
                "Everyone" => {
                    CheckResult::warn(Category::Privacy, "AirDrop", "AirDrop is set to Everyone")
                        .with_weight(3)
                        .with_fix_hint(
                            "Set to 'Contacts Only' in Control Center or System Settings",
                        )
                }
                other => CheckResult::pass(
                    Category::Privacy,
                    "AirDrop",
                    &format!("AirDrop mode: {}", other),
                )
                .with_weight(3),
            }
        }
        Err(_) => {
            // Not configured = likely default (Contacts Only on modern macOS)
            CheckResult::pass(
                Category::Privacy,
                "AirDrop",
                "AirDrop using default settings",
            )
            .with_weight(3)
        }
    }
}

fn check_user_launch_agents() -> CheckResult {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let path = format!("{}/Library/LaunchAgents", home);
    let dir = Path::new(&path);

    if !dir.exists() {
        return CheckResult::pass(
            Category::Privacy,
            "User Launch Agents",
            "No user Launch Agents directory",
        )
        .with_weight(0);
    }

    match fs::read_dir(dir) {
        Ok(entries) => {
            let agents: Vec<String> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|name| name.ends_with(".plist"))
                .collect();

            if agents.is_empty() {
                CheckResult::pass(
                    Category::Privacy,
                    "User Launch Agents",
                    "No user Launch Agents installed",
                )
                .with_weight(0)
            } else {
                let detail = agents
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let suffix = if agents.len() > 5 {
                    format!(" (+{} more)", agents.len() - 5)
                } else {
                    String::new()
                };

                CheckResult::pass(
                    Category::Privacy,
                    "User Launch Agents",
                    &format!("{} agent(s): {}{}", agents.len(), detail, suffix),
                )
                .with_weight(0)
                .with_detail("Review these for any unexpected or suspicious entries")
            }
        }
        Err(_) => CheckResult::skip(
            Category::Privacy,
            "User Launch Agents",
            "Could not read Launch Agents directory",
        ),
    }
}

fn check_system_launch_daemons() -> CheckResult {
    let dir = Path::new("/Library/LaunchDaemons");

    if !dir.exists() {
        return CheckResult::pass(
            Category::Privacy,
            "Third-Party Launch Daemons",
            "No third-party Launch Daemons directory",
        )
        .with_weight(0);
    }

    match fs::read_dir(dir) {
        Ok(entries) => {
            let non_apple: Vec<String> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|name| name.ends_with(".plist") && !name.starts_with("com.apple."))
                .collect();

            if non_apple.is_empty() {
                CheckResult::pass(
                    Category::Privacy,
                    "Third-Party Launch Daemons",
                    "No third-party Launch Daemons found",
                )
                .with_weight(0)
            } else {
                let detail = non_apple
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let suffix = if non_apple.len() > 5 {
                    format!(" (+{} more)", non_apple.len() - 5)
                } else {
                    String::new()
                };

                CheckResult::pass(
                    Category::Privacy,
                    "Third-Party Launch Daemons",
                    &format!("{} daemon(s): {}{}", non_apple.len(), detail, suffix),
                )
                .with_weight(0)
                .with_detail("Review these for any unexpected or suspicious entries")
            }
        }
        Err(_) => CheckResult::skip(
            Category::Privacy,
            "Third-Party Launch Daemons",
            "Could not read Launch Daemons directory",
        ),
    }
}

fn check_analytics_sharing() -> CheckResult {
    // Check if diagnostic data sharing is enabled
    match run_command(
        "defaults",
        &[
            "read",
            "/Library/Application Support/CrashReporter/DiagnosticMessagesHistory.plist",
            "AutoSubmit",
        ],
    ) {
        Ok(output) => {
            if output.trim() == "0" {
                CheckResult::pass(
                    Category::Privacy,
                    "Analytics Sharing",
                    "Diagnostic data sharing is disabled",
                )
                .with_weight(3)
            } else {
                CheckResult::warn(
                    Category::Privacy,
                    "Analytics Sharing",
                    "Diagnostic data sharing is enabled",
                )
                .with_weight(3)
                .with_fix_hint("Disable in System Settings > Privacy & Security > Analytics & Improvements")
            }
        }
        Err(_) => {
            // Can't determine -- skip
            CheckResult::skip(
                Category::Privacy,
                "Analytics Sharing",
                "Could not determine analytics sharing status",
            )
        }
    }
}
