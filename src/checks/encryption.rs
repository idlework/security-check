use crate::check::{Category, CheckResult};
use crate::runner::run_command;

pub fn run_checks() -> Vec<CheckResult> {
    vec![check_filevault()]
}

fn check_filevault() -> CheckResult {
    match run_command("fdesetup", &["status"]) {
        Ok(output) if output.contains("On") => {
            CheckResult::pass(Category::Encryption, "FileVault", "Disk encryption is on")
                .with_weight(10)
        }
        Ok(_) => {
            CheckResult::fail(Category::Encryption, "FileVault", "Disk encryption is off")
                .with_weight(10)
                .with_fix_hint("Enable in System Settings > Privacy & Security > FileVault")
        }
        Err(e) => CheckResult::skip(
            Category::Encryption,
            "FileVault",
            &format!("Could not check: {}", e),
        ),
    }
}
