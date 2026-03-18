use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsagePace {
    OnTrack,
    Ahead,
    Behind,
    Unknown,
}

/// Calculate usage pace based on current usage and time elapsed in the window.
pub fn calculate_pace(used_percent: f64, elapsed_percent: f64) -> UsagePace {
    if used_percent <= 0.0 || elapsed_percent <= 0.0 {
        return UsagePace::Unknown;
    }

    let expected_percent = elapsed_percent;
    let ratio = used_percent / expected_percent;

    if ratio > 1.2 {
        UsagePace::Ahead
    } else if ratio < 0.8 {
        UsagePace::Behind
    } else {
        UsagePace::OnTrack
    }
}
