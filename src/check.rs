use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Warn,
    Fail,
    Skip,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Pass => write!(f, "PASS"),
            Status::Warn => write!(f, "WARN"),
            Status::Fail => write!(f, "FAIL"),
            Status::Skip => write!(f, "SKIP"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    SystemProtection,
    Encryption,
    Firewall,
    MalwareProtection,
    SoftwareUpdates,
    Network,
    Hardware,
    UserSecurity,
    Privacy,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::SystemProtection => "System Protection",
            Category::Encryption => "Encryption",
            Category::Firewall => "Firewall",
            Category::MalwareProtection => "Malware Protection",
            Category::SoftwareUpdates => "Software Updates",
            Category::Network => "Network",
            Category::Hardware => "Hardware",
            Category::UserSecurity => "User Security",
            Category::Privacy => "Privacy",
        }
    }

    pub fn all() -> &'static [Category] {
        &[
            Category::SystemProtection,
            Category::Encryption,
            Category::Firewall,
            Category::MalwareProtection,
            Category::SoftwareUpdates,
            Category::Network,
            Category::Hardware,
            Category::UserSecurity,
            Category::Privacy,
        ]
    }

    pub fn matches_filter(&self, filter: &str) -> bool {
        self.label().to_lowercase().replace(' ', "_").contains(filter)
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub fn count_by_status(results: &[CheckResult]) -> (usize, usize, usize, usize) {
    let mut passed = 0;
    let mut warned = 0;
    let mut failed = 0;
    let mut skipped = 0;
    for r in results {
        match r.status {
            Status::Pass => passed += 1,
            Status::Warn => warned += 1,
            Status::Fail => failed += 1,
            Status::Skip => skipped += 1,
        }
    }
    (passed, warned, failed, skipped)
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub category: Category,
    pub name: String,
    pub status: Status,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
    #[serde(skip)]
    pub weight: u32,
}

impl CheckResult {
    fn new(status: Status, category: Category, name: &str, message: &str) -> Self {
        let weight = if status == Status::Skip { 0 } else { 5 };
        Self {
            category,
            name: name.to_string(),
            status,
            message: message.to_string(),
            detail: None,
            fix_hint: None,
            weight,
        }
    }

    pub fn pass(category: Category, name: &str, message: &str) -> Self {
        Self::new(Status::Pass, category, name, message)
    }

    pub fn warn(category: Category, name: &str, message: &str) -> Self {
        Self::new(Status::Warn, category, name, message)
    }

    pub fn fail(category: Category, name: &str, message: &str) -> Self {
        Self::new(Status::Fail, category, name, message)
    }

    pub fn skip(category: Category, name: &str, message: &str) -> Self {
        Self::new(Status::Skip, category, name, message)
    }

    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    pub fn with_fix_hint(mut self, hint: &str) -> Self {
        self.fix_hint = Some(hint.to_string());
        self
    }
}
