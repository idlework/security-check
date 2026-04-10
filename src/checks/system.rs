use crate::check::{Category, CheckResult};
use crate::runner::run_command;

pub fn run_checks(hw_output: &str) -> Vec<CheckResult> {
    vec![
        check_sip(),
        check_gatekeeper(),
        check_secure_boot(hw_output),
        check_custom_certificates(),
    ]
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

fn check_secure_boot(hw_output: &str) -> CheckResult {
    let boot_mode = hw_output
        .lines()
        .find(|l| l.contains("Secure Boot"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim());

    match boot_mode {
        Some(s) if s.contains("Full") => {
            CheckResult::pass(
                Category::SystemProtection,
                "Secure Boot",
                "Full Security mode",
            )
            .with_weight(8)
        }
        Some(s) => {
            CheckResult::warn(
                Category::SystemProtection,
                "Secure Boot",
                &format!("Secure Boot: {}", s),
            )
            .with_weight(8)
            .with_fix_hint("Set Full Security in Recovery Mode > Startup Security Utility")
        }
        None => CheckResult::skip(
            Category::SystemProtection,
            "Secure Boot",
            "Not available (Intel Mac or could not determine)",
        ),
    }
}

fn check_custom_certificates() -> CheckResult {
    // Check for user-added trust settings (custom CA certificates)
    match run_command("security", &["dump-trust-settings", "-d"]) {
        Ok(output) => {
            let count = output
                .lines()
                .filter(|l| l.starts_with("Cert "))
                .count();

            if count == 0 {
                CheckResult::pass(
                    Category::SystemProtection,
                    "Custom Certificates",
                    "No user-added trust settings",
                )
                .with_weight(5)
            } else {
                CheckResult::warn(
                    Category::SystemProtection,
                    "Custom Certificates",
                    &format!("{} custom certificate(s) in trust store", count),
                )
                .with_weight(5)
                .with_fix_hint("Review custom CAs in Keychain Access > System > Certificates")
                .with_detail("Custom CAs could indicate corporate proxy or MITM configuration")
            }
        }
        Err(e) if e.contains("No Trust Settings") || e.contains("SecTrustSettings") => {
            CheckResult::pass(
                Category::SystemProtection,
                "Custom Certificates",
                "No custom trust settings found",
            )
            .with_weight(5)
        }
        Err(_) => CheckResult::skip(
            Category::SystemProtection,
            "Custom Certificates",
            "Could not check trust store",
        ),
    }
}
