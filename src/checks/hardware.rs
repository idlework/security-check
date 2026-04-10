use crate::check::{Category, CheckResult};
use crate::runner::run_command;

pub fn run_checks() -> Vec<CheckResult> {
    let hw_output = run_command("system_profiler", &["SPHardwareDataType"]).unwrap_or_default();
    vec![
        check_system_info(&hw_output),
        check_activation_lock(&hw_output),
    ]
}

fn extract_field<'a>(output: &'a str, field: &str) -> Option<&'a str> {
    output
        .lines()
        .find(|l| l.contains(field))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim())
}

fn check_system_info(hw_output: &str) -> CheckResult {
    let model = extract_field(hw_output, "Model Name").unwrap_or("Unknown");
    let chip = extract_field(hw_output, "Chip")
        .or_else(|| extract_field(hw_output, "Processor Name"))
        .unwrap_or("Unknown");
    let memory = extract_field(hw_output, "Memory").unwrap_or("Unknown");
    let serial: String = extract_field(hw_output, "Serial Number")
        .map(|s| {
            let chars: Vec<char> = s.chars().collect();
            if chars.len() > 4 {
                format!("...{}", chars[chars.len() - 4..].iter().collect::<String>())
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| "Unknown".into());

    CheckResult::pass(
        Category::Hardware,
        "System Info",
        &format!("{}, {}, {}, Serial {}", model, chip, memory, serial),
    )
    .with_weight(0) // informational only
}

fn check_activation_lock(hw_output: &str) -> CheckResult {
    match extract_field(hw_output, "Activation Lock Status") {
        Some(s) if s.contains("Enabled") => {
            CheckResult::pass(Category::Hardware, "Activation Lock", "Activation Lock is enabled")
                .with_weight(5)
        }
        Some(_) => {
            CheckResult::warn(Category::Hardware, "Activation Lock", "Activation Lock is disabled")
                .with_weight(5)
                .with_fix_hint("Enable Find My Mac in System Settings > Apple ID > iCloud > Find My Mac")
        }
        None => CheckResult::skip(
            Category::Hardware,
            "Activation Lock",
            "Could not determine Activation Lock status",
        ),
    }
}
