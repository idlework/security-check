use crate::check::{Category, CheckResult};
use crate::runner::{run_command_timeout, run_defaults_read};
use std::time::Duration;

const SW_UPDATE_DOMAIN: &str = "/Library/Preferences/com.apple.SoftwareUpdate";

pub fn run_checks() -> Vec<CheckResult> {
    vec![
        check_auto_check(),
        check_auto_download(),
        check_auto_install(),
        check_critical_updates(),
        check_pending_updates(),
    ]
}

fn check_defaults_bool(key: &str, check_name: &str, pass_msg: &str, warn_msg: &str) -> CheckResult {
    match run_defaults_read(SW_UPDATE_DOMAIN, key) {
        Ok(output) => {
            let val = output.trim();
            if val == "1" {
                CheckResult::pass(Category::SoftwareUpdates, check_name, pass_msg).with_weight(5)
            } else {
                CheckResult::warn(Category::SoftwareUpdates, check_name, warn_msg)
                    .with_weight(5)
                    .with_fix_hint("Enable in System Settings > General > Software Update > Automatic Updates")
            }
        }
        Err(_) => {
            // Key not set -- macOS defaults may vary by version
            CheckResult::warn(Category::SoftwareUpdates, check_name, &format!("{} (not configured)", warn_msg))
                .with_weight(5)
                .with_fix_hint("Enable in System Settings > General > Software Update > Automatic Updates")
        }
    }
}

fn check_auto_check() -> CheckResult {
    check_defaults_bool(
        "AutomaticCheckEnabled",
        "Auto-Check Updates",
        "Automatic update check is enabled",
        "Automatic update check is disabled",
    )
}

fn check_auto_download() -> CheckResult {
    check_defaults_bool(
        "AutomaticDownload",
        "Auto-Download Updates",
        "Automatic download is enabled",
        "Automatic download is disabled",
    )
}

fn check_auto_install() -> CheckResult {
    check_defaults_bool(
        "AutomaticallyInstallMacOSUpdates",
        "Auto-Install macOS Updates",
        "Automatic macOS install is enabled",
        "Automatic macOS install is disabled",
    )
}

fn check_critical_updates() -> CheckResult {
    check_defaults_bool(
        "CriticalUpdateInstall",
        "Critical Update Install",
        "Critical/security updates auto-install is enabled",
        "Critical/security updates auto-install is disabled",
    )
}

fn check_pending_updates() -> CheckResult {
    // softwareupdate --list can be slow, use longer timeout
    match run_command_timeout("softwareupdate", &["--list"], Duration::from_secs(30)) {
        Ok(output) => {
            if output.contains("No new software available") {
                CheckResult::pass(
                    Category::SoftwareUpdates,
                    "Pending Updates",
                    "No pending updates",
                )
                .with_weight(5)
            } else {
                let update_count = output.lines().filter(|l| l.trim_start().starts_with('*')).count();
                if update_count > 0 {
                    CheckResult::warn(
                        Category::SoftwareUpdates,
                        "Pending Updates",
                        &format!("{} update(s) available", update_count),
                    )
                    .with_weight(5)
                    .with_fix_hint("Run: softwareupdate --install --all")
                } else {
                    CheckResult::pass(
                        Category::SoftwareUpdates,
                        "Pending Updates",
                        "No pending updates",
                    )
                    .with_weight(5)
                }
            }
        }
        Err(e) => {
            if e.contains("No new software available") {
                CheckResult::pass(
                    Category::SoftwareUpdates,
                    "Pending Updates",
                    "No pending updates",
                )
                .with_weight(5)
            } else {
                CheckResult::skip(
                    Category::SoftwareUpdates,
                    "Pending Updates",
                    &format!("Could not check: {}", e.chars().take(60).collect::<String>()),
                )
            }
        }
    }
}
