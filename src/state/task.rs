use std::cmp::Ordering;

use chrono::Local;
use icalendar::{Component, DatePerhapsTime, Todo, TodoStatus};

use chrono::NaiveDateTime;

use crate::state::utils::{dpt_to_naive_datetime, format_date_perhaps_time, get_naive_date};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub summary: String,
    pub completed: bool,
    /// True when the task has a start date that is in the future
    pub upcoming: bool,
    pub duetxt: String,
    pub due: Option<DatePerhapsTime>,
    pub start: Option<DatePerhapsTime>,
}

impl TaskItem {
    pub fn new(task: &Todo) -> Self {
        let completed = task.get_completed().is_some()
            || matches!(task.get_status(), Some(TodoStatus::Completed));
        let summary = task.get_summary().unwrap_or("Untitled Task").to_string();
        let due = task.get_due();
        let duetxt = match &due {
            Some(d) => format_date_perhaps_time(d),
            None => "No due date".to_string(),
        };
        let start = task.get_start();
        let upcoming = start.as_ref().is_some_and(|s| {
            let today = Local::now().date_naive();
            match s {
                DatePerhapsTime::Date(d) => *d > today,
                DatePerhapsTime::DateTime(cdt) => get_naive_date(cdt) > today,
            }
        });
        Self {
            completed,
            upcoming,
            summary,
            duetxt,
            due,
            start,
        }
    }
}

impl PartialOrd for TaskItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TaskItem {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_key = self
            .due
            .as_ref()
            .map_or(NaiveDateTime::MAX, dpt_to_naive_datetime);
        let other_key = other
            .due
            .as_ref()
            .map_or(NaiveDateTime::MAX, dpt_to_naive_datetime);
        self_key.cmp(&other_key)
    }
}
