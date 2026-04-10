use crate::check::{Category, CheckResult};
use crate::runner::run_command;

const SOCKETFILTERFW: &str = "/usr/libexec/ApplicationFirewall/socketfilterfw";

pub fn run_checks() -> Vec<CheckResult> {
    vec![check_firewall(), check_stealth_mode(), check_logging()]
}

fn check_firewall() -> CheckResult {
    match run_command(SOCKETFILTERFW, &["--getglobalstate"]) {
        Ok(output) => {
            if output.contains("enabled") || output.contains("State = 1") {
                CheckResult::pass(Category::Firewall, "Firewall", "Firewall is enabled")
                    .with_weight(10)
            } else {
                CheckResult::fail(Category::Firewall, "Firewall", "Firewall is disabled")
                    .with_weight(10)
                    .with_fix_hint("Enable in System Settings > Network > Firewall")
            }
        }
        Err(e) => CheckResult::skip(
            Category::Firewall,
            "Firewall",
            &format!("Could not check: {}", e),
        ),
    }
}

fn check_stealth_mode() -> CheckResult {
    match run_command(SOCKETFILTERFW, &["--getstealthmode"]) {
        Ok(output) => {
            if output.contains("enabled") {
                CheckResult::pass(Category::Firewall, "Stealth Mode", "Stealth mode is enabled")
                    .with_weight(3)
            } else {
                CheckResult::warn(Category::Firewall, "Stealth Mode", "Stealth mode is disabled")
                    .with_weight(3)
                    .with_fix_hint("Run: sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setstealthmode on")
            }
        }
        Err(e) => CheckResult::skip(
            Category::Firewall,
            "Stealth Mode",
            &format!("Could not check: {}", e),
        ),
    }
}

fn check_logging() -> CheckResult {
    match run_command(SOCKETFILTERFW, &["--getloggingmode"]) {
        Ok(output) => {
            if output.contains("enabled") || output.contains("Log mode is on") {
                CheckResult::pass(Category::Firewall, "Firewall Logging", "Logging is enabled")
                    .with_weight(3)
            } else {
                CheckResult::warn(Category::Firewall, "Firewall Logging", "Logging is disabled")
                    .with_weight(3)
                    .with_fix_hint("Run: sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setloggingmode on")
            }
        }
        Err(e) => CheckResult::skip(
            Category::Firewall,
            "Firewall Logging",
            &format!("Could not check: {}", e),
        ),
    }
}
