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
    pub const ALL: [EditorField; 5] = [
        EditorField::Summary,
        EditorField::Start,
        EditorField::End,
        EditorField::Location,
        EditorField::Description,
    ];

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

    pub fn next(&self, down: bool) -> Self {
        let len = Self::ALL.len();
        let idx = Self::ALL.iter().position(|f| f == self).unwrap();
        let next_idx = if down {
            (idx + 1) % len
        } else if idx == 0 {
            len - 1
        } else {
            idx - 1
        };
        Self::ALL[next_idx]
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
            EditItem::Event(event) => (
                event.summary.clone(),
                event.start.as_ref().map_or("".to_string(), dpt_to_str),
                event.end.as_ref().map_or("".to_string(), dpt_to_str),
                event.details.location.clone(),
                event.details.description.clone(),
            ),
            EditItem::Task(task) => (
                task.summary.clone(),
                task.start.as_ref().map_or("".to_string(), dpt_to_str),
                task.due.as_ref().map_or("".to_string(), dpt_to_str),
                task.details.location.clone(),
                task.details.description.clone(),
            ),
        };
        Self {
            selected_field: (EditorField::Summary, false),
            summary,
            start,
            end,
            location: location.unwrap_or_default(),
            desc: desc.unwrap_or_default(),
            item,
        }
    }

    pub fn event(e: EventItem) -> Self {
        let (location, desc) = e.details.to_strs();
        Self {
            selected_field: (EditorField::Summary, false),
            summary: e.summary.clone(),
            start: e.start.as_ref().map_or("".to_string(), dpt_to_str),
            end: e.end.as_ref().map_or("".to_string(), dpt_to_str),
            location,
            desc,
            item: EditItem::Event(e),
        }
    }

    pub fn is_editing(&self) -> bool {
        self.selected_field.1
    }

    pub fn get_field_value(&self, field: EditorField) -> &str {
        match field {
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
