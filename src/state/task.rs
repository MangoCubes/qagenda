use icalendar::{Component, DatePerhapsTime, Todo, TodoStatus};

use crate::state::utils::format_date_perhaps_time;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskItem {
    pub summary: String,
    pub completed: bool,
    pub due: String,
    pub start: Option<DatePerhapsTime>,
}

impl TaskItem {
    pub fn new(task: &Todo) -> Self {
        let completed = task.get_completed().is_some()
            || matches!(task.get_status(), Some(TodoStatus::Completed));
        let summary = task.get_summary().unwrap_or("Untitled Task").to_string();
        let due = match task.get_due() {
            Some(d) => format_date_perhaps_time(&d),
            None => "No due date".to_string(),
        };
        Self {
            completed,
            summary,
            due,
            start: task.get_start(),
        }
    }
}
