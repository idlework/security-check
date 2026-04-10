use crate::check::{CheckResult, Status};

pub fn calculate_score(results: &[CheckResult]) -> (u32, u32) {
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
            Status::Fail => {}
            Status::Skip => {}
        }
    }

    (earned, possible)
}

pub fn grade(pct: u32) -> &'static str {
    match pct {
        95..=100 => "A+",
        90..=94 => "A",
        85..=89 => "B+",
        80..=84 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
}
