use crate::check::{Category, CheckResult};
use crate::runner::{run_command, Context};

pub fn run_checks(ctx: &Context) -> Vec<CheckResult> {
    let services = run_command("launchctl", &["list"]).unwrap_or_default();

    vec![
        check_ssh(ctx),
        check_sharing_service(&services, "com.apple.smbd", "File Sharing", "File Sharing"),
        check_sharing_service(&services, "com.apple.screensharing", "Screen Sharing", "Screen Sharing"),
        check_remote_management(),
        check_dns(),
    ]
}

fn check_ssh(ctx: &Context) -> CheckResult {
    if let Ok(output) = run_command("systemsetup", &["-getremotelogin"]) {
        if output.contains("Off") {
            return CheckResult::pass(Category::Network, "Remote Login (SSH)", "SSH is disabled")
                .with_weight(5);
        } else if output.contains("On") {
            return CheckResult::warn(Category::Network, "Remote Login (SSH)", "SSH is enabled")
                .with_weight(5)
                .with_fix_hint("Disable in System Settings > General > Sharing > Remote Login");
        }
    }

    if !ctx.is_root {
        return CheckResult::skip(
            Category::Network,
            "Remote Login (SSH)",
            "Requires sudo to check",
        );
    }

    match run_command("lsof", &["-Pni", "TCP:22"]) {
        Ok(output) if output.contains("LISTEN") => {
            CheckResult::warn(Category::Network, "Remote Login (SSH)", "SSH is listening on port 22")
                .with_weight(5)
                .with_fix_hint("Disable in System Settings > General > Sharing > Remote Login")
        }
        _ => CheckResult::pass(Category::Network, "Remote Login (SSH)", "SSH is not listening")
            .with_weight(5),
    }
}

fn check_sharing_service(
    services: &str,
    service_id: &str,
    name: &str,
    settings_name: &str,
) -> CheckResult {
    let is_running = services.lines().any(|l| l.contains(service_id));

    if is_running {
        CheckResult::warn(Category::Network, name, &format!("{} is active", name))
            .with_weight(5)
            .with_fix_hint(&format!(
                "Disable in System Settings > General > Sharing > {}",
                settings_name
            ))
    } else {
        CheckResult::pass(Category::Network, name, &format!("{} is disabled", name))
            .with_weight(5)
    }
}

fn check_remote_management() -> CheckResult {
    match run_command(
        "defaults",
        &["read", "/Library/Preferences/com.apple.RemoteManagement", "ARD_AllLocalUsers"],
    ) {
        Ok(output) if output.trim() == "1" => {
            CheckResult::warn(Category::Network, "Remote Management", "Apple Remote Desktop is enabled")
                .with_weight(5)
                .with_fix_hint("Disable in System Settings > General > Sharing > Remote Management")
        }
        _ => CheckResult::pass(Category::Network, "Remote Management", "Remote management is not configured")
            .with_weight(5),
    }
}

fn check_dns() -> CheckResult {
    let service = find_active_network_service().unwrap_or_else(|| "Wi-Fi".to_string());

    match run_command("networksetup", &["-getdnsservers", &service]) {
        Ok(output) if output.contains("There aren't any DNS Servers") => {
            CheckResult::warn(
                Category::Network,
                "DNS Configuration",
                "Using default ISP DNS (no custom DNS set)",
            )
            .with_weight(3)
            .with_fix_hint("Consider using a privacy-focused DNS like 1.1.1.1 or 9.9.9.9")
        }
        Ok(output) => {
            let servers: Vec<&str> = output.trim().lines().take(3).collect();
            CheckResult::pass(
                Category::Network,
                "DNS Configuration",
                &format!("Custom DNS: {}", servers.join(", ")),
            )
            .with_weight(3)
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
