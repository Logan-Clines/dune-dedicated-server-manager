use std::str::FromStr;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use cron::Schedule as CronSchedule;

use super::timezone;

#[derive(Debug, Clone)]
pub enum Schedule {
    /// Fire every `every` from the moment the scheduler starts.
    Interval { every: Duration },
    /// Fire daily at `hour:minute` in the configured IANA timezone.
    Daily { hour: u32, minute: u32 },
    /// Fire on the cadence described by a (5-, 6-, or 7-field) cron
    /// expression, evaluated in the operator's TZ.
    Cron(Box<CronSchedule>),
    /// Never fire automatically. Manual triggers still work.
    Disabled,
}

/// Parses a user-supplied cron expression. Only the standard 5-field form
/// (`min hour dom mon dow`) is accepted; the underlying parser wants 6 fields
/// (seconds first) so we prepend `0` seconds before handing it off. Anything
/// other than exactly 5 fields is rejected with a clear error.
pub fn parse_cron(expr: &str) -> Result<CronSchedule> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("empty cron expression"));
    }
    let field_count = trimmed.split_whitespace().count();
    if field_count != 5 {
        return Err(anyhow!(
            "cron must have exactly 5 fields (min hour dom mon dow); got {field_count}"
        ));
    }
    let normalized = format!("0 {trimmed}");
    CronSchedule::from_str(&normalized).map_err(|err| anyhow!("invalid cron expression: {err}"))
}

impl Schedule {
    pub fn interval_secs(secs: u64) -> Self {
        if secs == 0 {
            Self::Disabled
        } else {
            Self::Interval {
                every: Duration::from_secs(secs),
            }
        }
    }

    pub fn daily(hour: u32, minute: u32) -> Self {
        Self::Daily { hour, minute }
    }

    pub fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }

    pub fn next_fire(&self, tz: Tz, now: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Self::Interval { every } => {
                now + chrono::Duration::from_std(*every).expect("interval fits in chrono::Duration")
            }
            Self::Daily { hour, minute } => timezone::next_daily_at(tz, *hour, *minute, now),
            Self::Cron(schedule) => {
                let now_tz = now.with_timezone(&tz);
                schedule
                    .after(&now_tz)
                    .next()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|| now + chrono::Duration::days(365 * 100))
            }
            // Sentinel "very far future" so the loop sleeps until cancellation
            // even if a caller forgets to check `is_disabled`.
            Self::Disabled => now + chrono::Duration::days(365 * 100),
        }
    }

    pub fn describe(&self, tz: Tz) -> String {
        match self {
            Self::Interval { every } => format!("every {}s", every.as_secs()),
            Self::Daily { hour, minute } => {
                format!("daily {:02}:{:02} {}", hour, minute, tz.name())
            }
            Self::Cron(schedule) => format!("cron `{schedule}` {}", tz.name()),
            Self::Disabled => "disabled (manual-only)".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cron_accepts_5_fields() {
        let s = parse_cron("0 4 * * *").expect("parse");
        // Sanity: should produce a valid CronSchedule
        let next: Vec<_> = s.upcoming(chrono::Utc).take(2).collect();
        assert_eq!(next.len(), 2);
    }

    #[test]
    fn parse_cron_rejects_bad_field_count() {
        assert!(parse_cron("1 2 3 4").is_err());
        assert!(parse_cron("").is_err());
        assert!(parse_cron("not cron at all").is_err());
        // 6-field with seconds is also rejected; we keep the surface 5-only.
        assert!(parse_cron("0 0 4 * * *").is_err());
        // 7-field with year is also rejected.
        assert!(parse_cron("0 0 4 * * * 2026").is_err());
    }

    #[test]
    fn parse_cron_rejects_invalid_field() {
        assert!(parse_cron("99 4 * * *").is_err());
    }
}
