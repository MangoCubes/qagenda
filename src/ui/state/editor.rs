use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use icalendar::{CalendarDateTime, DatePerhapsTime};

use crate::state::{event::EventItem, task::TaskItem, utils::get_naive_datetime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFieldType {
    Year,
    Month,
    Day,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeFieldType {
    Year,
    Month,
    Day,
    Hour,
    Minute,
}

/// Used for storing both the value of that field as well as the field that is currently being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeField {
    None,
    Date(NaiveDate, DateFieldType),
    DateTime(NaiveDateTime, DateTimeFieldType),
}

impl TimeField {
    pub fn to_dpt(&self) -> Option<DatePerhapsTime> {
        match self {
            TimeField::None => None,
            TimeField::Date(nd, _) => Some(DatePerhapsTime::Date(*nd)),
            TimeField::DateTime(ndt, _) => {
                Some(DatePerhapsTime::DateTime(CalendarDateTime::WithTimezone {
                    date_time: *ndt,
                    tzid: iana_time_zone::get_timezone().unwrap_or_default(),
                }))
            }
        }
    }
}

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
    pub orig: EditItem,
    pub is_new: bool,
    pub selected_field: (EditorField, bool),
    pub summary: String,
    pub start: TimeField,
    pub end: TimeField,
    pub location: String,
    pub desc: String,
}

impl EditorState {
    pub fn next(&mut self, down: bool) {
        todo!()
    }

    pub fn new(item: EditItem, is_new: bool) -> Self {
        fn convert(orig: &Option<DatePerhapsTime>) -> TimeField {
            if let Some(t) = orig {
                match t {
                    DatePerhapsTime::DateTime(cdt) => {
                        TimeField::DateTime(get_naive_datetime(&cdt), DateTimeFieldType::Year)
                    }
                    DatePerhapsTime::Date(nd) => TimeField::Date(*nd, DateFieldType::Year),
                }
            } else {
                TimeField::None
            }
        }
        let (summary, start, end, location, desc) = match &item {
            EditItem::Event(event) => (
                event.summary.clone(),
                convert(&event.start),
                convert(&event.end),
                event.details.location.clone(),
                event.details.description.clone(),
            ),
            EditItem::Task(task) => (
                task.summary.clone(),
                convert(&task.start),
                convert(&task.due),
                task.details.location.clone(),
                task.details.description.clone(),
            ),
        };
        Self {
            selected_field: (EditorField::Summary, false),
            is_new,
            summary,
            start,
            end,
            location: location.unwrap_or_default(),
            desc: desc.unwrap_or_default(),
            orig: item,
        }
    }

    pub fn new_event(cal: String) -> Self {
        Self::new(EditItem::Event(EventItem::create(cal)), true)
    }

    pub fn new_task(cal: String) -> Self {
        Self::new(EditItem::Task(TaskItem::create(cal)), true)
    }

    pub fn is_editing(&self) -> bool {
        self.selected_field.1
    }

    pub fn set_field_value(&mut self, value: String) -> Result<(), String> {
        let update_time = |time: &mut TimeField| -> Result<(), String> {
            let Ok(v) = value.parse() else {
                return Err("Invalid number!".into());
            };
            match time {
                TimeField::None => unreachable!(),
                TimeField::Date(nd, t) => {
                    let Some(new) = (match t {
                        DateFieldType::Year => nd.with_year(v),
                        DateFieldType::Month => nd.with_month(v as u32),
                        DateFieldType::Day => nd.with_day(v as u32),
                    }) else {
                        return Err("Invalid value!".into());
                    };
                    Ok(*nd = new)
                }
                TimeField::DateTime(ndt, t) => {
                    let Some(new) = (match t {
                        DateTimeFieldType::Year => ndt.with_year(v),
                        DateTimeFieldType::Month => ndt.with_month(v as u32),
                        DateTimeFieldType::Day => ndt.with_day(v as u32),
                        DateTimeFieldType::Hour => ndt.with_hour(v as u32),
                        DateTimeFieldType::Minute => ndt.with_minute(v as u32),
                    }) else {
                        return Err("Invalid value!".into());
                    };
                    Ok(*ndt = new)
                }
            }
        };
        match self.selected_field.0 {
            EditorField::Summary => Ok(self.summary = value),
            EditorField::Start => update_time(&mut self.start),
            EditorField::End => update_time(&mut self.end),
            EditorField::Location => Ok(self.location = value),
            EditorField::Description => Ok(self.desc = value),
        }
    }
}
