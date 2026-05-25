use std::time::Duration;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use super::timezone;

#[derive(Debug, Clone, Copy)]
pub enum Schedule {
    /// Fire every `every` from the moment the scheduler starts.
    Interval { every: Duration },
    /// Fire daily at `hour:minute` in the configured IANA timezone.
    Daily { hour: u32, minute: u32 },
}

impl Schedule {
    pub fn interval_secs(secs: u64) -> Self {
        Self::Interval {
            every: Duration::from_secs(secs),
        }
    }

    pub fn daily(hour: u32, minute: u32) -> Self {
        Self::Daily { hour, minute }
    }

    pub fn next_fire(&self, tz: Tz, now: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Self::Interval { every } => {
                now + chrono::Duration::from_std(*every).expect("interval fits in chrono::Duration")
            }
            Self::Daily { hour, minute } => timezone::next_daily_at(tz, *hour, *minute, now),
        }
    }

    pub fn describe(&self, tz: Tz) -> String {
        match self {
            Self::Interval { every } => format!("every {}s", every.as_secs()),
            Self::Daily { hour, minute } => {
                format!("daily {:02}:{:02} {}", hour, minute, tz.name())
            }
        }
    }
}
