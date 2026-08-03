use crate::state::{event::EventItem, task::TaskItem, utils::dpt_to_str};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditItem {
    Event(EventItem),
    Task(TaskItem),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorField {
    Summary,
    Start,
    End,
    Location,
    Description,
}

impl EditorField {
    pub fn label(&self, is_event: bool) -> &'static str {
        match self {
            EditorField::Summary => "Summary",
            EditorField::Start => "Start",
            EditorField::End => {
                if is_event {
                    "End"
                } else {
                    "Due"
                }
            }
            EditorField::Location => "Location",
            EditorField::Description => "Description",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorState {
    pub item: EditItem,
    pub selected_field: (EditorField, bool),
    pub summary: String,
    pub start: String,
    pub end: String,
    pub location: String,
    pub desc: String,
}

impl EditorState {
    pub fn new(item: EditItem) -> Self {
        let (summary, start, end, location, desc) = match &item {
            EditItem::Event(event) => {
                let (l, d) = match &event.details {
                    Some(d) => d.to_strs(),
                    None => ("".to_string(), "".to_string()),
                };
                (
                    event.summary.clone(),
                    event.start.as_ref().map_or("".to_string(), dpt_to_str),
                    event.end.as_ref().map_or("".to_string(), dpt_to_str),
                    l,
                    d,
                )
            }
            EditItem::Task(task) => {
                let (l, d) = match &task.details {
                    Some(d) => d.to_strs(),
                    None => ("".to_string(), "".to_string()),
                };
                (
                    task.summary.clone(),
                    task.start.as_ref().map_or("".to_string(), dpt_to_str),
                    task.due.as_ref().map_or("".to_string(), dpt_to_str),
                    l,
                    d,
                )
            }
        };
        Self {
            selected_field: (EditorField::Summary, false),
            summary,
            start,
            end,
            location,
            desc,
            item,
        }
    }
    pub fn event(e: EventItem) -> Self {
        Self {
            selected_field: (EditorField::Summary, false),
            summary: e.summary.clone(),
            start: e.start.as_ref().map_or("".to_string(), dpt_to_str),
            end: e.end.as_ref().map_or("".to_string(), dpt_to_str),
            location: e
                .details
                .as_ref()
                .and_then(|d| d.location.clone())
                .unwrap_or_default(),
            desc: e
                .details
                .as_ref()
                .and_then(|d| d.description.clone())
                .unwrap_or_default(),
            item: EditItem::Event(e),
        }
    }

    pub fn get_field_value(&self) -> &str {
        match self.selected_field.0 {
            EditorField::Summary => &self.summary,
            EditorField::Start => &self.start,
            EditorField::End => &self.end,
            EditorField::Location => &self.location,
            EditorField::Description => &self.desc,
        }
    }

    pub fn set_field_value(&mut self, value: String) {
        match self.selected_field.0 {
            EditorField::Summary => self.summary = value,
            EditorField::Start => self.start = value,
            EditorField::End => self.end = value,
            EditorField::Location => self.location = value,
            EditorField::Description => self.desc = value,
        }
    }
}
