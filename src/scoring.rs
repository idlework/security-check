use crate::check::{CheckResult, Status};

pub struct Score {
    pub earned: u32,
    pub possible: u32,
}

impl Score {
    pub fn from_results(results: &[CheckResult]) -> Self {
        let mut earned = 0u32;
        let mut possible = 0u32;

        for result in results {
            if result.weight == 0 || result.status == Status::Skip {
                continue;
            }
            possible += result.weight;
            match result.status {
                Status::Pass => earned += result.weight,
                Status::Warn => earned += result.weight / 2,
                Status::Fail | Status::Skip => {}
            }
        }

        Self { earned, possible }
    }

    pub fn percentage(&self) -> u32 {
        if self.possible > 0 {
            (self.earned as f64 / self.possible as f64 * 100.0).round() as u32
        } else {
            0
        }
    }

    pub fn grade(&self) -> &'static str {
        match self.percentage() {
            95..=100 => "A+",
            90..=94 => "A",
            85..=89 => "B+",
            80..=84 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        }
    }
}
