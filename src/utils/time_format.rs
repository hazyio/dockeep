use serde::{Deserialize, Serialize};

/// Controls how times are rendered in human-readable date strings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum TimeFormat {
    /// 24-hour clock, e.g. `14:30`.
    #[default]
    TwentyFourHour,
    /// 12-hour clock with AM/PM suffix, e.g. `2:30 PM`.
    TwelveHour,
}

impl TimeFormat {
    const ALL: &'static [TimeFormat] = &[TimeFormat::TwentyFourHour, TimeFormat::TwelveHour];

    pub fn all() -> &'static [TimeFormat] {
        Self::ALL
    }

    /// Short identifier used as the dropdown value key.
    pub fn id(&self) -> &'static str {
        match self {
            TimeFormat::TwentyFourHour => "24h",
            TimeFormat::TwelveHour => "12h",
        }
    }

    /// Human-readable label shown in the UI.
    pub fn label(&self) -> &'static str {
        match self {
            TimeFormat::TwentyFourHour => "24-hour (14:30)",
            TimeFormat::TwelveHour => "12-hour (2:30 PM)",
        }
    }

    /// Format a `(hour, minute)` pair according to this format.
    pub fn format_time(&self, hour: u32, minute: u32) -> String {
        match self {
            TimeFormat::TwentyFourHour => format!("{:02}:{:02}", hour, minute),
            TimeFormat::TwelveHour => {
                let period = if hour < 12 { "AM" } else { "PM" };
                let h = match hour % 12 {
                    0 => 12,
                    h => h,
                };
                format!("{:02}:{:02} {}", h, minute, period)
            }
        }
    }
}
