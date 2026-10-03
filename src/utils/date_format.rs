use serde::{Deserialize, Serialize};

/// Controls how the date portion is rendered in human-readable date strings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum DateFormat {
    /// Day/Month/Year, e.g. `31/12/2026`.
    #[default]
    DayMonthYear,
    /// Month/Day/Year, e.g. `12/31/2026`.
    MonthDayYear,
    /// Year-Month-Day (ISO 8601 / sortable), e.g. `2026-12-31`.
    YearMonthDay,
}

impl DateFormat {
    const ALL: &'static [DateFormat] = &[
        DateFormat::DayMonthYear,
        DateFormat::MonthDayYear,
        DateFormat::YearMonthDay,
    ];

    pub fn all() -> &'static [DateFormat] {
        Self::ALL
    }

    /// Short identifier used as the dropdown value key.
    pub fn id(&self) -> &'static str {
        match self {
            DateFormat::DayMonthYear => "dmy",
            DateFormat::MonthDayYear => "mdy",
            DateFormat::YearMonthDay => "ymd",
        }
    }

    /// Human-readable label shown in the UI.
    pub fn label(&self) -> &'static str {
        match self {
            DateFormat::DayMonthYear => "Day/Month/Year (31/12/2026)",
            DateFormat::MonthDayYear => "Month/Day/Year (12/31/2026)",
            DateFormat::YearMonthDay => "Year-Month-Day (2026-12-31)",
        }
    }

    /// Format a `(year, month, day)` triple according to this format.
    pub fn format_date(&self, year: i32, month: u32, day: u32) -> String {
        match self {
            DateFormat::DayMonthYear => format!("{:02}/{:02}/{}", day, month, year),
            DateFormat::MonthDayYear => format!("{:02}/{:02}/{}", month, day, year),
            DateFormat::YearMonthDay => format!("{}-{:02}-{:02}", year, month, day),
        }
    }
}
