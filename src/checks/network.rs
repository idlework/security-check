use crate::check::{Category, CheckResult};
use crate::runner::{run_command, Context};

pub fn run_checks(ctx: &Context) -> Vec<CheckResult> {
    vec![
        check_ssh(ctx),
        check_file_sharing(),
        check_screen_sharing(),
        check_remote_management(),
        check_dns(),
    ]
}

fn check_ssh(ctx: &Context) -> CheckResult {
    // Try systemsetup first (most reliable)
    match run_command("systemsetup", &["-getremotelogin"]) {
        Ok(output) => {
            if output.contains("Off") {
                return CheckResult::pass(Category::Network, "Remote Login (SSH)", "SSH is disabled")
                    .with_weight(5);
            } else if output.contains("On") {
                return CheckResult::warn(Category::Network, "Remote Login (SSH)", "SSH is enabled")
                    .with_weight(5)
                    .with_fix_hint("Disable in System Settings > General > Sharing > Remote Login");
            }
        }
        Err(_) => {}
    }

    // Fallback: check if sshd is listening (requires sudo for lsof)
    if !ctx.is_root {
        return CheckResult::skip(
            Category::Network,
            "Remote Login (SSH)",
            "Requires sudo to check",
        );
    }

    match run_command("lsof", &["-Pni", "TCP:22"]) {
        Ok(output) => {
            if output.contains("LISTEN") {
                CheckResult::warn(Category::Network, "Remote Login (SSH)", "SSH is listening on port 22")
                    .with_weight(5)
                    .with_fix_hint("Disable in System Settings > General > Sharing > Remote Login")
            } else {
                CheckResult::pass(Category::Network, "Remote Login (SSH)", "SSH is not listening")
                    .with_weight(5)
            }
        }
        Err(_) => CheckResult::pass(Category::Network, "Remote Login (SSH)", "SSH is not listening")
            .with_weight(5),
    }
}

fn check_file_sharing() -> CheckResult {
    match run_command("launchctl", &["list"]) {
        Ok(output) => {
            let smb_running = output.lines().any(|l| l.contains("com.apple.smbd"));

            if smb_running {
                CheckResult::warn(Category::Network, "File Sharing", "File sharing (SMB) is active")
                    .with_weight(5)
                    .with_fix_hint("Disable in System Settings > General > Sharing > File Sharing")
            } else {
                CheckResult::pass(Category::Network, "File Sharing", "File sharing is disabled")
                    .with_weight(5)
            }
        }
        Err(_) => CheckResult::skip(
            Category::Network,
            "File Sharing",
            "Could not check sharing services",
        ),
    }
}

fn check_screen_sharing() -> CheckResult {
    match run_command("launchctl", &["list"]) {
        Ok(output) => {
            let screen_sharing = output
                .lines()
                .any(|l| l.contains("com.apple.screensharing"));

            if screen_sharing {
                CheckResult::warn(
                    Category::Network,
                    "Screen Sharing",
                    "Screen sharing is active",
                )
                .with_weight(5)
                .with_fix_hint("Disable in System Settings > General > Sharing > Screen Sharing")
            } else {
                CheckResult::pass(
                    Category::Network,
                    "Screen Sharing",
                    "Screen sharing is disabled",
                )
                .with_weight(5)
            }
        }
        Err(_) => CheckResult::skip(
            Category::Network,
            "Screen Sharing",
            "Could not check sharing services",
        ),
    }
}

fn check_remote_management() -> CheckResult {
    match run_command(
        "defaults",
        &["read", "/Library/Preferences/com.apple.RemoteManagement", "ARD_AllLocalUsers"],
    ) {
        Ok(output) => {
            let val = output.trim();
            if val == "1" {
                CheckResult::warn(
                    Category::Network,
                    "Remote Management",
                    "Apple Remote Desktop is enabled",
                )
                .with_weight(5)
                .with_fix_hint("Disable in System Settings > General > Sharing > Remote Management")
            } else {
                CheckResult::pass(
                    Category::Network,
                    "Remote Management",
                    "Remote management is disabled",
                )
                .with_weight(5)
            }
        }
        Err(_) => {
            // Domain not found = not configured = good
            CheckResult::pass(
                Category::Network,
                "Remote Management",
                "Remote management is not configured",
            )
            .with_weight(5)
        }
    }
}

fn check_dns() -> CheckResult {
    // Find the primary network service
    let service = find_active_network_service().unwrap_or_else(|| "Wi-Fi".to_string());

    match run_command("networksetup", &["-getdnsservers", &service]) {
        Ok(output) => {
            let trimmed = output.trim();
            if trimmed.contains("There aren't any DNS Servers") {
                CheckResult::warn(
                    Category::Network,
                    "DNS Configuration",
                    "Using default ISP DNS (no custom DNS set)",
                )
                .with_weight(3)
                .with_fix_hint("Consider using a privacy-focused DNS like 1.1.1.1 or 9.9.9.9")
            } else {
                let servers: Vec<&str> = trimmed.lines().take(3).collect();
                CheckResult::pass(
                    Category::Network,
                    "DNS Configuration",
                    &format!("Custom DNS: {}", servers.join(", ")),
                )
                .with_weight(3)
            }
        }
        Err(_) => CheckResult::skip(
            Category::Network,
            "DNS Configuration",
            "Could not check DNS settings",
        ),
    }
}

fn find_active_network_service() -> Option<String> {
    let output = run_command("networksetup", &["-listallnetworkservices"]).ok()?;
    output
        .lines()
        .skip(1) // Skip "An asterisk..." header
        .find(|l| !l.starts_with('*') && !l.is_empty())
        .map(|s| s.to_string())
}
