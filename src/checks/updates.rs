use crate::check::{Category, CheckResult};
use crate::runner::{run_command_timeout, run_defaults_read};
use std::time::Duration;

const SW_UPDATE_DOMAIN: &str = "/Library/Preferences/com.apple.SoftwareUpdate";

pub fn run_checks() -> Vec<CheckResult> {
    vec![
        check_defaults_bool(
            "AutomaticCheckEnabled",
            "Auto-Check Updates",
            "Automatic update check",
        ),
        check_defaults_bool(
            "AutomaticDownload",
            "Auto-Download Updates",
            "Automatic download",
        ),
        check_defaults_bool(
            "AutomaticallyInstallMacOSUpdates",
            "Auto-Install macOS Updates",
            "Automatic macOS install",
        ),
        check_defaults_bool(
            "CriticalUpdateInstall",
            "Critical Update Install",
            "Critical/security updates auto-install",
        ),
        check_pending_updates(),
    ]
}

fn check_defaults_bool(key: &str, check_name: &str, label: &str) -> CheckResult {
    match run_defaults_read(SW_UPDATE_DOMAIN, key) {
        Ok(output) if output.trim() == "1" => {
            CheckResult::pass(Category::SoftwareUpdates, check_name, &format!("{} is enabled", label))
                .with_weight(5)
        }
        _ => {
            CheckResult::warn(Category::SoftwareUpdates, check_name, &format!("{} is disabled", label))
                .with_weight(5)
                .with_fix_hint("Enable in System Settings > General > Software Update > Automatic Updates")
        }
    }
}

fn check_pending_updates() -> CheckResult {
    let output = match run_command_timeout("softwareupdate", &["--list"], Duration::from_secs(30)) {
        Ok(o) => o,
        Err(e) => {
            return CheckResult::skip(
                Category::SoftwareUpdates,
                "Pending Updates",
                &format!("Could not check: {}", e.chars().take(60).collect::<String>()),
            );
        }
    };

    if output.contains("No new software available") {
        return CheckResult::pass(Category::SoftwareUpdates, "Pending Updates", "No pending updates")
            .with_weight(5);
    }

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
        CheckResult::pass(Category::SoftwareUpdates, "Pending Updates", "No pending updates")
            .with_weight(5)
    }
}
