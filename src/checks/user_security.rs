use crate::check::{Category, CheckResult};
use crate::runner::{run_command, run_defaults_read};

pub fn run_checks() -> Vec<CheckResult> {
    vec![
        check_screen_lock(),
        check_screensaver_password(),
        check_guest_account(),
        check_auto_login(),
        check_mdm_enrollment(),
    ]
}

fn check_screen_lock() -> CheckResult {
    // sysadminctl writes to stderr -- runner combines stdout+stderr
    match run_command("sysadminctl", &["-screenLock", "status"]) {
        Ok(output) if output.contains("screenLock is off") => {
            CheckResult::warn(Category::UserSecurity, "Screen Lock", "Screen lock is disabled")
                .with_weight(5)
                .with_fix_hint("Enable in System Settings > Lock Screen")
        }
        Ok(output) if output.contains("immediate") => {
            CheckResult::pass(Category::UserSecurity, "Screen Lock", "Screen lock activates immediately")
                .with_weight(5)
        }
        Ok(_) => {
            CheckResult::warn(Category::UserSecurity, "Screen Lock", "Screen lock has a delay before activation")
                .with_weight(5)
                .with_fix_hint("Set to 'Immediately' in System Settings > Lock Screen")
        }
        Err(_) => CheckResult::skip(
            Category::UserSecurity,
            "Screen Lock",
            "Could not determine screen lock status",
        ),
    }
}

fn check_screensaver_password() -> CheckResult {
    match run_defaults_read("com.apple.screensaver", "askForPassword") {
        Ok(output) if output.trim() == "1" => {
            CheckResult::pass(Category::UserSecurity, "Screensaver Password", "Password required after screensaver")
                .with_weight(5)
        }
        _ => {
            CheckResult::warn(Category::UserSecurity, "Screensaver Password", "Screensaver password not configured")
                .with_weight(5)
                .with_fix_hint("Enable in System Settings > Lock Screen")
        }
    }
}

fn check_guest_account() -> CheckResult {
    match run_defaults_read("/Library/Preferences/com.apple.loginwindow", "GuestEnabled") {
        Ok(output) if output.trim() == "1" => {
            CheckResult::warn(Category::UserSecurity, "Guest Account", "Guest account is enabled")
                .with_weight(5)
                .with_fix_hint("Disable in System Settings > Users & Groups > Guest User")
        }
        _ => {
            // Not set or "0" = disabled (default on modern macOS)
            CheckResult::pass(Category::UserSecurity, "Guest Account", "Guest account is disabled")
                .with_weight(5)
        }
    }
}

fn check_auto_login() -> CheckResult {
    match run_defaults_read("/Library/Preferences/com.apple.loginwindow", "autoLoginUser") {
        Ok(output) if !output.contains("does not exist") => {
            CheckResult::fail(
                Category::UserSecurity,
                "Auto-Login",
                &format!("Auto-login is enabled for '{}'", output.trim()),
            )
            .with_weight(5)
            .with_fix_hint("Disable in System Settings > Users & Groups > Automatic Login")
        }
        _ => {
            CheckResult::pass(Category::UserSecurity, "Auto-Login", "Auto-login is disabled")
                .with_weight(5)
        }
    }
}

fn check_mdm_enrollment() -> CheckResult {
    // profiles writes to stderr -- runner combines stdout+stderr
    match run_command("profiles", &["status", "-type", "enrollment"]) {
        Ok(output) => {
            let mdm = output.contains("MDM enrollment: Yes");
            let dep = output.contains("Enrolled via DEP: Yes");

            let msg = match (mdm, dep) {
                (true, true) => "Enrolled via DEP with MDM",
                (true, false) => "MDM enrolled (not via DEP)",
                (false, true) => "DEP enrolled (no MDM)",
                (false, false) => "Not enrolled in MDM",
            };

            CheckResult::pass(Category::UserSecurity, "MDM Enrollment", msg).with_weight(0)
        }
        Err(_) => CheckResult::skip(
            Category::UserSecurity,
            "MDM Enrollment",
            "Could not check enrollment status",
        ),
    }
}
