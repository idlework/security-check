use crate::check::{Category, CheckResult};
use crate::runner::run_command;

pub fn run_checks() -> Vec<CheckResult> {
    vec![check_sip(), check_gatekeeper()]
}

fn check_sip() -> CheckResult {
    match run_command("csrutil", &["status"]) {
        Ok(output) if output.contains("enabled") => {
            CheckResult::pass(Category::SystemProtection, "System Integrity Protection", "SIP is enabled")
                .with_weight(10)
        }
        Ok(_) => {
            CheckResult::fail(Category::SystemProtection, "System Integrity Protection", "SIP is disabled")
                .with_weight(10)
                .with_fix_hint("Reboot into Recovery Mode and run: csrutil enable")
        }
        Err(e) => CheckResult::skip(
            Category::SystemProtection,
            "System Integrity Protection",
            &format!("Could not check: {}", e),
        ),
    }
}

fn check_gatekeeper() -> CheckResult {
    // spctl writes to stderr even on success -- runner combines stdout+stderr
    match run_command("spctl", &["--status"]) {
        Ok(output) if output.contains("assessments enabled") => {
            CheckResult::pass(Category::SystemProtection, "Gatekeeper", "Assessments enabled")
                .with_weight(10)
        }
        Ok(_) => {
            CheckResult::fail(Category::SystemProtection, "Gatekeeper", "Gatekeeper is disabled")
                .with_weight(10)
                .with_fix_hint("Run: sudo spctl --master-enable")
        }
        Err(e) => CheckResult::skip(
            Category::SystemProtection,
            "Gatekeeper",
            &format!("Could not check: {}", e),
        ),
    }
}
