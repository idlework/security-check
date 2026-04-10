use crate::check::{Category, CheckResult};
use crate::runner::run_command;

pub fn run_checks() -> Vec<CheckResult> {
    let hw_output = run_command("system_profiler", &["SPHardwareDataType"]).unwrap_or_default();
    vec![
        check_system_info(&hw_output),
        check_activation_lock(&hw_output),
    ]
}

fn extract_field(output: &str, field: &str) -> Option<String> {
    output
        .lines()
        .find(|l| l.contains(field))
        .map(|l| l.split(':').skip(1).collect::<Vec<&str>>().join(":").trim().to_string())
}

fn check_system_info(hw_output: &str) -> CheckResult {
    let model = extract_field(hw_output, "Model Name").unwrap_or_else(|| "Unknown".into());
    let chip = extract_field(hw_output, "Chip").unwrap_or_else(|| {
        extract_field(hw_output, "Processor Name").unwrap_or_else(|| "Unknown".into())
    });
    let memory = extract_field(hw_output, "Memory").unwrap_or_else(|| "Unknown".into());
    let serial = extract_field(hw_output, "Serial Number")
        .map(|s| {
            if s.len() > 4 {
                format!("...{}", &s[s.len() - 4..])
            } else {
                s
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
        Some(status) => {
            if status.contains("Enabled") {
                CheckResult::pass(
                    Category::Hardware,
                    "Activation Lock",
                    "Activation Lock is enabled",
                )
                .with_weight(5)
            } else {
                CheckResult::warn(
                    Category::Hardware,
                    "Activation Lock",
                    "Activation Lock is disabled",
                )
                .with_weight(5)
                .with_fix_hint("Enable Find My Mac in System Settings > Apple ID > iCloud > Find My Mac")
            }
        }
        None => CheckResult::skip(
            Category::Hardware,
            "Activation Lock",
            "Could not determine Activation Lock status",
        ),
    }
}
