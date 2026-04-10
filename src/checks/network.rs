use crate::check::{truncate_list, Category, CheckResult};
use crate::runner::{run_command, Context};
use std::fs;
use std::path::Path;

pub fn run_checks(ctx: &Context) -> Vec<CheckResult> {
    let services = run_command("launchctl", &["list"]).unwrap_or_default();

    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

    vec![
        check_ssh(ctx),
        check_sharing_service(&services, "com.apple.smbd", "File Sharing"),
        check_sharing_service(&services, "com.apple.screensharing", "Screen Sharing"),
        check_remote_management(),
        check_dns(),
        check_listening_ports(),
        check_ssh_keys(&home),
        check_authorized_keys(&home),
        check_vpn(),
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

fn check_sharing_service(services: &str, service_id: &str, name: &str) -> CheckResult {
    if services.lines().any(|l| l.contains(service_id)) {
        CheckResult::warn(Category::Network, name, &format!("{} is active", name))
            .with_weight(5)
            .with_fix_hint(&format!(
                "Disable in System Settings > General > Sharing > {}",
                name
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

fn check_listening_ports() -> CheckResult {
    match run_command("lsof", &["-iTCP", "-sTCP:LISTEN", "-nP"]) {
        Ok(output) => {
            const KNOWN_SAFE: &[&str] = &[
                "rapportd",
                "ControlCe",
                "UserEvent",
                "SystemUIServer",
                "sharingd",
                "remoted",
                "WiFiAgent",
            ];

            let mut listeners: Vec<String> = Vec::new();
            for line in output.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() < 9 {
                    continue;
                }
                let process = fields[0];
                let addr = fields[8];
                let port = addr.rsplit(':').next().unwrap_or("?");
                let entry = format!("{}:{}", process, port);
                if !listeners.contains(&entry) {
                    listeners.push(entry);
                }
            }

            let unexpected: Vec<&String> = listeners
                .iter()
                .filter(|l| !KNOWN_SAFE.iter().any(|s| l.starts_with(s)))
                .collect();

            if listeners.is_empty() {
                CheckResult::pass(
                    Category::Network,
                    "Listening Ports",
                    "No TCP listeners found",
                )
                .with_weight(3)
            } else if unexpected.is_empty() {
                CheckResult::pass(
                    Category::Network,
                    "Listening Ports",
                    &format!("{} listener(s), all known services", listeners.len()),
                )
                .with_weight(3)
            } else {
                let unexpected_strs: Vec<&str> = unexpected.iter().map(|s| s.as_str()).collect();
                CheckResult::warn(
                    Category::Network,
                    "Listening Ports",
                    &format!("{} unexpected: {}", unexpected.len(), truncate_list(&unexpected_strs, 5)),
                )
                .with_weight(3)
                .with_detail(&listeners.join(", "))
            }
        }
        Err(_) => CheckResult::skip(
            Category::Network,
            "Listening Ports",
            "Could not check listening ports",
        ),
    }
}

fn check_ssh_keys(home: &str) -> CheckResult {
    let ssh_dir = Path::new(home).join(".ssh");
    if !ssh_dir.exists() {
        return CheckResult::pass(Category::Network, "SSH Keys", "No SSH directory found")
            .with_weight(3);
    }

    let pub_keys: Vec<String> = match fs::read_dir(&ssh_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".pub"))
            .filter_map(|e| {
                let content = fs::read_to_string(e.path()).ok()?;
                let key_type = content.split_whitespace().next()?;
                Some(format!(
                    "{} ({})",
                    e.file_name().to_string_lossy(),
                    key_type.trim_start_matches("ssh-")
                ))
            })
            .collect(),
        Err(_) => {
            return CheckResult::skip(
                Category::Network,
                "SSH Keys",
                "Could not read ~/.ssh directory",
            )
        }
    };

    if pub_keys.is_empty() {
        return CheckResult::pass(Category::Network, "SSH Keys", "No SSH public keys found")
            .with_weight(3);
    }

    // Check for weak key types
    let has_dsa = pub_keys.iter().any(|k| k.contains("(dss)") || k.contains("(dsa)"));
    if has_dsa {
        return CheckResult::warn(
            Category::Network,
            "SSH Keys",
            &format!("{} key(s) found, includes weak DSA key", pub_keys.len()),
        )
        .with_weight(3)
        .with_fix_hint("Replace DSA keys with Ed25519: ssh-keygen -t ed25519")
        .with_detail(&pub_keys.join(", "));
    }

    CheckResult::pass(
        Category::Network,
        "SSH Keys",
        &format!("{} key(s): {}", pub_keys.len(), pub_keys.join(", ")),
    )
    .with_weight(3)
}

fn check_authorized_keys(home: &str) -> CheckResult {
    let auth_keys = Path::new(home).join(".ssh/authorized_keys");

    if !auth_keys.exists() {
        return CheckResult::pass(
            Category::Network,
            "Authorized Keys",
            "No authorized_keys file",
        )
        .with_weight(3);
    }

    match fs::read_to_string(&auth_keys) {
        Ok(content) => {
            let count = content
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .count();
            if count == 0 {
                CheckResult::pass(
                    Category::Network,
                    "Authorized Keys",
                    "authorized_keys is empty",
                )
                .with_weight(3)
            } else {
                CheckResult::warn(
                    Category::Network,
                    "Authorized Keys",
                    &format!("{} authorized key(s) allow remote access", count),
                )
                .with_weight(3)
                .with_fix_hint("Review ~/.ssh/authorized_keys for unexpected entries")
            }
        }
        Err(_) => CheckResult::skip(
            Category::Network,
            "Authorized Keys",
            "Could not read authorized_keys",
        ),
    }
}

fn check_vpn() -> CheckResult {
    match run_command("scutil", &["--nc", "list"]) {
        Ok(output) => {
            let vpns: Vec<&str> = output
                .lines()
                .filter(|l| l.contains("(Disconnected)") || l.contains("(Connected)"))
                .collect();

            if vpns.is_empty() {
                CheckResult::warn(
                    Category::Network,
                    "VPN Configuration",
                    "No VPN configured",
                )
                .with_weight(3)
                .with_fix_hint("Consider setting up a VPN for network privacy")
            } else {
                let connected = vpns.iter().filter(|l| l.contains("(Connected)")).count();
                let msg = if connected > 0 {
                    format!("{} VPN(s) configured, {} connected", vpns.len(), connected)
                } else {
                    format!("{} VPN(s) configured, none active", vpns.len())
                };
                CheckResult::pass(Category::Network, "VPN Configuration", &msg).with_weight(3)
            }
        }
        Err(_) => CheckResult::skip(
            Category::Network,
            "VPN Configuration",
            "Could not check VPN configuration",
        ),
    }
}

fn find_active_network_service() -> Option<String> {
    // Get the default route's network interface (e.g. "en0")
    let route_output = run_command("route", &["-n", "get", "default"]).ok()?;
    let interface = route_output
        .lines()
        .find(|l| l.contains("interface:"))?
        .split(':')
        .nth(1)?
        .trim();

    // Map interface to network service name
    let services = run_command("networksetup", &["-listallhardwareports"]).ok()?;
    let mut current_service = None;
    for line in services.lines() {
        if let Some(name) = line.strip_prefix("Hardware Port: ") {
            current_service = Some(name.to_string());
        } else if line.contains("Device:") && line.contains(interface) {
            return current_service;
        }
    }

    // Fallback to Wi-Fi
    Some("Wi-Fi".to_string())
}
