use std::cmp::Ordering;

use chrono::{Days, Local};
use icalendar::{Component, DatePerhapsTime, Event, EventLike};

use crate::state::utils::{
    dpt_to_naive_datetime, format_date_perhaps_time, format_time_only, get_naive_date,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventItem {
    pub summary: String,
    pub duration: String,
    pub start: Option<DatePerhapsTime>,
    pub end: Option<DatePerhapsTime>,
    pub location: Option<String>,
    pub description: Option<String>,
}

impl EventItem {
    fn new(
        summary: String,
        start: Option<DatePerhapsTime>,
        end: Option<DatePerhapsTime>,
        location: Option<String>,
        description: Option<String>,
    ) -> Self {
        let duration = match (&start, &end) {
            (Some(DatePerhapsTime::DateTime(s)), Some(DatePerhapsTime::DateTime(e))) => {
                if get_naive_date(&s) == get_naive_date(&e) {
                    // Same day, different end time
                    format!(
                        "{} - {}",
                        format_date_perhaps_time(&start.as_ref().unwrap()),
                        format_time_only(&e)
                    )
                } else {
                    format!(
                        "{} - {}",
                        format_date_perhaps_time(&start.as_ref().unwrap()),
                        format_date_perhaps_time(&end.as_ref().unwrap())
                    )
                }
            }
            (Some(s), Some(e)) => {
                let s_str = format_date_perhaps_time(&s);
                if let (DatePerhapsTime::Date(sd), DatePerhapsTime::Date(ed)) = (s, e)
                    && sd.checked_add_days(Days::new(1)).unwrap() == *ed
                {
                    format!("{}", s_str)
                } else {
                    // Leaving every other cases as is for now
                    let e_str = format_date_perhaps_time(&e);
                    if s_str == e_str {
                        s_str
                    } else {
                        format!("{} - {}", s_str, e_str)
                    }
                }
            }
            (Some(s), None) => format!("Since {}", format_date_perhaps_time(&s)),
            (None, Some(e)) => format!("Until {}", format_date_perhaps_time(&e)),
            (None, None) => "No time set".to_string(),
        };
        Self {
            summary,
            duration,
            start,
            end,
            location,
            description,
        }
    }

    pub fn from(event: &Event) -> Self {
        Self::new(
            event.get_summary().unwrap_or("Untitled Event").to_string(),
            event.get_start(),
            event.get_end(),
            event.get_location().map(str::to_string),
            event.get_description().map(str::to_string),
        )
    }

    pub fn with_custom_time(
        event: &Event,
        start: DatePerhapsTime,
        end: Option<DatePerhapsTime>,
    ) -> Self {
        Self::new(
            event.get_summary().unwrap_or("Untitled Event").to_string(),
            Some(start),
            end,
            event.get_location().map(str::to_string),
            event.get_description().map(str::to_string),
        )
    }

    pub fn in_progress(&self) -> bool {
        let now = Local::now().naive_local();
        let started = self
            .start
            .as_ref()
            .is_some_and(|s| dpt_to_naive_datetime(s) <= now);
        let not_ended = self
            .end
            .as_ref()
            .map_or(true, |e| dpt_to_naive_datetime(e) > now);
        started && not_ended
    }
}

impl PartialOrd for EventItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EventItem {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_in_progress = self.in_progress();
        let other_in_progress = other.in_progress();

        match (self_in_progress, other_in_progress) {
            // Both events are in progress
            // Event that is closer to end is prioritised
            (true, true) => {
                let self_key = self.end.as_ref().map(dpt_to_naive_datetime);
                let other_key = other.end.as_ref().map(dpt_to_naive_datetime);
                self_key.cmp(&other_key)
            }
            // Always prioritise tasks that are currently in progress
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            // Otherwise, sort by their start time
            (false, false) => {
                let self_key = self.start.as_ref().map(dpt_to_naive_datetime);
                let other_key = other.start.as_ref().map(dpt_to_naive_datetime);
                self_key.cmp(&other_key)
            }
        }
    }
}
