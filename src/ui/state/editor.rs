use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
use icalendar::{CalendarDateTime, DatePerhapsTime};

use crate::{
    config::keybinds::Direction,
    state::{event::EventItem, task::TaskItem, utils::get_naive_datetime},
};

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

    fn cycle(&mut self, next: bool) {
        match self {
            TimeField::None => (),
            TimeField::Date(_, date_field_type) => {
                *date_field_type = if next {
                    match date_field_type {
                        DateFieldType::Year => DateFieldType::Month,
                        DateFieldType::Month => DateFieldType::Day,
                        DateFieldType::Day => DateFieldType::Year,
                    }
                } else {
                    match date_field_type {
                        DateFieldType::Year => DateFieldType::Day,
                        DateFieldType::Month => DateFieldType::Year,
                        DateFieldType::Day => DateFieldType::Month,
                    }
                }
            }
            TimeField::DateTime(_, date_time_field_type) => {
                *date_time_field_type = if next {
                    match date_time_field_type {
                        DateTimeFieldType::Year => DateTimeFieldType::Month,
                        DateTimeFieldType::Month => DateTimeFieldType::Day,
                        DateTimeFieldType::Day => DateTimeFieldType::Hour,
                        DateTimeFieldType::Hour => DateTimeFieldType::Minute,
                        DateTimeFieldType::Minute => DateTimeFieldType::Year,
                    }
                } else {
                    match date_time_field_type {
                        DateTimeFieldType::Year => DateTimeFieldType::Minute,
                        DateTimeFieldType::Month => DateTimeFieldType::Year,
                        DateTimeFieldType::Day => DateTimeFieldType::Month,
                        DateTimeFieldType::Hour => DateTimeFieldType::Day,
                        DateTimeFieldType::Minute => DateTimeFieldType::Hour,
                    }
                }
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
    pub fn next(&mut self, dir: &Direction) {
        match self.selected_field {
            (EditorField::Summary, false) => match dir {
                Direction::Up => self.selected_field.0 = EditorField::Description,
                Direction::Down => self.selected_field.0 = EditorField::Start,
                _ => (),
            },
            (EditorField::Start, false) => match dir {
                Direction::Up => self.selected_field.0 = EditorField::Summary,
                Direction::Down => self.selected_field.0 = EditorField::End,
                Direction::Right => self.start.cycle(true),
                Direction::Left => self.start.cycle(false),
            },
            (EditorField::End, false) => match dir {
                Direction::Up => self.selected_field.0 = EditorField::Start,
                Direction::Down => self.selected_field.0 = EditorField::Location,
                Direction::Right => self.end.cycle(true),
                Direction::Left => self.end.cycle(false),
            },
            (EditorField::Location, false) => match dir {
                Direction::Up => self.selected_field.0 = EditorField::End,
                Direction::Down => self.selected_field.0 = EditorField::Description,
                _ => (),
            },
            (EditorField::Description, false) => match dir {
                Direction::Up => self.selected_field.0 = EditorField::Location,
                Direction::Down => self.selected_field.0 = EditorField::Summary,
                _ => (),
            },
            _ => (),
        }
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

    pub fn edit_field(&mut self) {
        self.selected_field.1 = match self.selected_field.0 {
            EditorField::Start => !matches!(self.start, TimeField::None),
            EditorField::End => !matches!(self.end, TimeField::None),
            _ => true,
        };
    }

    pub fn cycle_time_field(&mut self) {
        match self.selected_field.0 {
            EditorField::Start => self.start = self.start.cycle_type(),
            EditorField::End => self.end = self.end.cycle_type(),
            _ => (),
        }
    }
}
