use icalendar::{Component, DatePerhapsTime, Event};

use crate::state::utils::{format_date_perhaps_time, format_time_only, get_naive_date};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventItem {
    pub summary: String,
    pub duration: String,
}
impl EventItem {
    pub fn new(event: &Event) -> Self {
        let summary = event.get_summary().unwrap_or("Untitled Event").to_string();
        let start = event.get_start();
        let end = event.get_end();
        let duration = match (event.get_start(), event.get_end()) {
            (Some(DatePerhapsTime::Date(s)), Some(DatePerhapsTime::Date(e))) => {
                format!("{} - {}", s.format("%Y-%m-%d"), e.format("%Y-%m-%d"))
            }
            (Some(DatePerhapsTime::DateTime(s)), Some(DatePerhapsTime::DateTime(e))) => {
                if get_naive_date(&s) == get_naive_date(&e) {
                    // Same day, different end time
                    format!(
                        "{} - {}",
                        format_date_perhaps_time(&start.unwrap()),
                        format_time_only(&e)
                    )
                } else {
                    format!(
                        "{} - {}",
                        format_date_perhaps_time(&start.unwrap()),
                        format_date_perhaps_time(&end.unwrap())
                    )
                }
            }
            (Some(s), Some(e)) => {
                // Leaving every other cases as is for now
                let s_str = format_date_perhaps_time(&s);
                let e_str = format_date_perhaps_time(&e);
                if s_str == e_str {
                    s_str
                } else {
                    format!("{} - {}", s_str, e_str)
                }
            }
            (Some(s), None) => format_date_perhaps_time(&s),
            (None, Some(e)) => format!("Until {}", format_date_perhaps_time(&e)),
            (None, None) => "No time set".to_string(),
        };
        Self { summary, duration }
    }
}
