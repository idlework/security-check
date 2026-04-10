use crate::check::{truncate_list, Category, CheckResult};
use crate::runner::{run_command, run_defaults_read};
use std::fs;
use std::path::Path;

pub fn run_checks() -> Vec<CheckResult> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

    vec![
        check_airdrop(),
        list_plists(
            &format!("{}/Library/LaunchAgents", home),
            "User Launch Agents",
            |_| true,
        ),
        list_plists(
            "/Library/LaunchDaemons",
            "Third-Party Launch Daemons",
            |name| !name.starts_with("com.apple."),
        ),
        check_login_items(),
        check_analytics_sharing(),
        check_tcc(&home, "Accessibility Access", "kTCCServiceAccessibility"),
        check_tcc(&home, "Screen Recording", "kTCCServiceScreenCapture"),
        check_tcc(&home, "Full Disk Access", "kTCCServiceSystemPolicyAllFiles"),
    ]
}

fn check_airdrop() -> CheckResult {
    match run_defaults_read("com.apple.sharingd", "DiscoverableMode") {
        Ok(output) => match output.trim() {
            "Off" => {
                CheckResult::pass(Category::Privacy, "AirDrop", "AirDrop is off").with_weight(3)
            }
            "Contacts Only" | "ContactsOnly" => {
                CheckResult::pass(Category::Privacy, "AirDrop", "AirDrop set to Contacts Only")
                    .with_weight(3)
            }
            "Everyone" => {
                CheckResult::warn(Category::Privacy, "AirDrop", "AirDrop is set to Everyone")
                    .with_weight(3)
                    .with_fix_hint("Set to 'Contacts Only' in Control Center or System Settings")
            }
            other => CheckResult::pass(
                Category::Privacy,
                "AirDrop",
                &format!("AirDrop mode: {}", other),
            )
            .with_weight(3),
        },
        Err(_) => {
            CheckResult::pass(Category::Privacy, "AirDrop", "AirDrop using default settings")
                .with_weight(3)
        }
    }
}

fn list_plists(dir_path: &str, name: &str, filter: fn(&str) -> bool) -> CheckResult {
    let dir = Path::new(dir_path);

    if !dir.exists() {
        return CheckResult::pass(Category::Privacy, name, &format!("No {} directory", name))
            .with_weight(0);
    }

    let entries: Vec<String> = match fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".plist") && filter(n))
            .collect(),
        Err(_) => {
            return CheckResult::skip(
                Category::Privacy,
                name,
                &format!("Could not read {} directory", name),
            )
        }
    };

    if entries.is_empty() {
        return CheckResult::pass(Category::Privacy, name, &format!("No {} found", name))
            .with_weight(0);
    }

    CheckResult::pass(
        Category::Privacy,
        name,
        &format!("{} item(s): {}", entries.len(), truncate_list(&entries, 5)),
    )
    .with_weight(0)
    .with_detail("Review these for any unexpected or suspicious entries")
}

fn check_login_items() -> CheckResult {
    match run_command(
        "osascript",
        &["-e", "tell application \"System Events\" to get the name of every login item"],
    ) {
        Ok(output) => {
            let items: Vec<&str> = output
                .trim()
                .split(", ")
                .filter(|s| !s.is_empty())
                .collect();

            if items.is_empty() {
                CheckResult::pass(Category::Privacy, "Login Items", "No login items configured")
                    .with_weight(0)
            } else {
                CheckResult::pass(
                    Category::Privacy,
                    "Login Items",
                    &format!("{} item(s): {}", items.len(), truncate_list(&items, 5)),
                )
                .with_weight(0)
                .with_detail("Review these for any unexpected applications")
            }
        }
        Err(_) => CheckResult::skip(
            Category::Privacy,
            "Login Items",
            "Could not query login items",
        ),
    }
}

fn check_analytics_sharing() -> CheckResult {
    match run_command(
        "defaults",
        &[
            "read",
            "/Library/Application Support/CrashReporter/DiagnosticMessagesHistory.plist",
            "AutoSubmit",
        ],
    ) {
        Ok(output) if output.trim() == "0" => {
            CheckResult::pass(Category::Privacy, "Analytics Sharing", "Diagnostic data sharing is disabled")
                .with_weight(3)
        }
        Ok(_) => {
            CheckResult::warn(Category::Privacy, "Analytics Sharing", "Diagnostic data sharing is enabled")
                .with_weight(3)
                .with_fix_hint("Disable in System Settings > Privacy & Security > Analytics & Improvements")
        }
        Err(_) => CheckResult::skip(
            Category::Privacy,
            "Analytics Sharing",
            "Could not determine analytics sharing status",
        ),
    }
}

fn check_tcc(home: &str, name: &str, service: &str) -> CheckResult {
    let user_db = format!(
        "{}/Library/Application Support/com.apple.TCC/TCC.db",
        home
    );

    let query = format!(
        "SELECT client FROM access WHERE service='{}' AND auth_value=2;",
        service
    );

    match run_command("sqlite3", &[&user_db, &query]) {
        Ok(output) if output.contains("Error:") || output.contains("authorization denied") => {
            CheckResult::skip(
                Category::Privacy,
                name,
                &format!("TCC database not accessible (requires Full Disk Access)"),
            )
        }
        Ok(output) => {
            let apps: Vec<&str> = output
                .lines()
                .filter(|l| !l.trim().is_empty())
                .collect();

            if apps.is_empty() {
                CheckResult::pass(
                    Category::Privacy,
                    name,
                    &format!("No apps with {} permission", name.to_lowercase()),
                )
                .with_weight(0)
            } else {
                CheckResult::pass(
                    Category::Privacy,
                    name,
                    &format!("{} app(s): {}", apps.len(), truncate_list(&apps, 5)),
                )
                .with_weight(0)
                .with_detail("Review these permissions in System Settings > Privacy & Security")
            }
        }
        Err(_) => CheckResult::skip(
            Category::Privacy,
            name,
            &format!("Could not query {} permissions", name.to_lowercase()),
        ),
    }
}
