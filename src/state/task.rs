use std::cmp::Ordering;
use std::path::PathBuf;

use chrono::Local;
use icalendar::{Component, DatePerhapsTime, EventLike, Todo, TodoStatus};

use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::state::details::Details;
use crate::state::diff::SingleDiff;
use crate::state::utils::{dpt_to_naive_datetime, format_date_perhaps_time, get_naive_date};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub cal: String,
    pub summary: String,
    pub completed: bool,
    /// True when the task has a start date that is in the future
    pub upcoming: bool,
    pub duetxt: String,
    pub due: Option<DatePerhapsTime>,
    pub start: Option<DatePerhapsTime>,
    pub details: Details,
    pub uid: String,
    pub path: PathBuf,
}

impl TaskItem {
    fn gen_duetxt(due: &Option<DatePerhapsTime>) -> String {
        match due {
            Some(d) => format_date_perhaps_time(d),
            None => "No due date".to_string(),
        }
    }
    pub fn rebuild(&mut self) {
        self.duetxt = Self::gen_duetxt(&self.due);
    }

    /// Create a new task from scratch
    /// The [`path`] variable is the path to the directory that contains all calendars
    pub fn create(path: &PathBuf, cal: String) -> Self {
        let uid = Uuid::new_v4().to_string();
        Self {
            path: path.join(&cal).join(format!("{}.ics", uid)),
            cal,
            summary: String::new(),
            completed: false,
            upcoming: false,
            duetxt: "No due date".to_string(),
            due: None,
            start: None,
            details: Details::new(None, None),
            uid,
        }
    }

    /// Create a [`TaskItem`] object from a [`Todo`] object
    /// The [`path`] variable is the path to the [`Todo`] item file
    pub fn new(path: PathBuf, cal: String, task: &Todo) -> Self {
        let completed = task.get_completed().is_some()
            || matches!(task.get_status(), Some(TodoStatus::Completed));
        let summary = task.get_summary().unwrap_or("Untitled Task").to_string();
        let due = task.get_due();
        let duetxt = Self::gen_duetxt(&due);
        let start = task.get_start();
        let upcoming = start.as_ref().is_some_and(|s| {
            let today = Local::now().date_naive();
            match s {
                DatePerhapsTime::Date(d) => *d > today,
                DatePerhapsTime::DateTime(cdt) => get_naive_date(cdt) > today,
            }
        });
        Self {
            cal,
            completed,
            upcoming,
            summary,
            duetxt,
            due,
            start,
            details: Details::new(
                task.get_location().map(str::to_string),
                task.get_description().map(str::to_string),
            ),
            uid: task
                .property_value("UID")
                .map(|s| s.to_string())
                .expect("Task without UID is not supported!"),
            path,
        }
    }

    /// [`other`] is the new task
    pub fn diff(&self, other: &TaskItem) -> SingleDiff {
        // For now, handling change in completed status only
        let mut changes = vec![];
        fn completed(val: bool) -> &'static str {
            if val { "complete" } else { "incomplete" }
        }
        if self.completed != other.completed {
            changes.push(format!(
                "Task {} -> {}",
                completed(self.completed),
                completed(other.completed),
            ));
        }
        SingleDiff::Update {
            summary: (if self.summary != other.summary {
                format!("{} (Renamed from \"{}\")", other.summary, self.summary)
            } else {
                self.summary.clone()
            }),
            changes,
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
