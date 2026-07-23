pub mod event;
pub mod task;
pub mod utils;

use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::PathBuf,
    str::FromStr,
};

use chrono::{Days, Local, NaiveTime, TimeDelta, TimeZone};
use icalendar::{Calendar, Component, DatePerhapsTime, EventLike, rrule::Tz};

use crate::{
    debug,
    state::{
        event::EventItem,
        task::TaskItem,
        utils::{get_naive_date, get_naive_datetime, is_past_event},
    },
};

#[derive(Clone)]
pub struct MiniCal {
    pub events: Vec<EventItem>,
    pub recurring_events: Vec<EventItem>,
    /// If the last occurrence of a recurring event is past the current date, it goes here as well
    pub past_events: Vec<EventItem>,
    pub tasks: Vec<TaskItem>,
    pub completed_tasks: Vec<TaskItem>,
    /// Tasks whose start date is in the future
    pub upcoming_tasks: Vec<TaskItem>,
}

impl MiniCal {
    pub fn from_calendar(
        cal: &Calendar,
        max_recurrence_count: u32,
        max_recurrence_date: u32,
    ) -> Self {
        let today = Local::now().date_naive();
        let (mut events, mut recurring_events, mut past_events) = (vec![], vec![], vec![]);
        let start_window = Tz::LOCAL
            .from_local_datetime(&today.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
            .single()
            .expect("Apparently the local time falls in a fold or a gap in the local time. At least that's what the documentation says. I have no idea what the hell that means. Sorry.");

        let end_window = if max_recurrence_date > 0 {
            Some(
                Tz::LOCAL
                    .from_local_datetime(
                        &today
                            .checked_add_days(Days::new((max_recurrence_date + 1) as u64))
                            .expect("Max recurrence date is too big!")
                            .and_time(NaiveTime::default()),
                    )
                    .single()
                    .expect("Failed to compute end window for recurrence date limit"),
            )
        } else {
            None
        };

        cal.events().for_each(|event| {
            if event.property_value("RRULE").is_some() {
                match event.get_recurrence() {
                    Ok(rrule) => {
                        let result = {
                            let after = rrule.after(start_window);
                            let bounded = match end_window {
                                Some(end) => after.before(end),
                                None => after,
                            };
                            bounded.all(max_recurrence_count as u16)
                        };
                        if result.dates.is_empty() {
                            // All occurrences are in the past
                            past_events.push(EventItem::from(event));
                        } else {
                            let items = match event.get_end() {
                                Some(end) => {
                                    let Some(start) = event.get_start() else {
                                        if is_past_event(event) {
                                            past_events.push(EventItem::from(event));
                                        } else {
                                            events.push(EventItem::from(event));
                                        };
                                        return;
                                    };
                                    let duration = {
                                        match (start, end) {
                                            (
                                                DatePerhapsTime::Date(s),
                                                DatePerhapsTime::Date(e),
                                            ) => e - s,
                                            (
                                                DatePerhapsTime::DateTime(s),
                                                DatePerhapsTime::DateTime(e),
                                            ) => get_naive_datetime(&e) - get_naive_datetime(&s),
                                            _ => TimeDelta::zero(),
                                        }
                                    };
                                    result
                                        .dates
                                        .iter()
                                        .map(|start| {
                                            let s = start.naive_local();
                                            EventItem::with_custom_time(
                                                event,
                                                s.into(),
                                                Some((s + duration).into()),
                                            )
                                        })
                                        .collect::<Vec<EventItem>>()
                                }
                                None => {
                                    // No end date? Huh
                                    result
                                        .dates
                                        .iter()
                                        .map(|start| {
                                            EventItem::with_custom_time(
                                                event,
                                                start.naive_local().into(),
                                                None,
                                            )
                                        })
                                        .collect::<Vec<EventItem>>()
                                }
                            };
                            recurring_events.extend(items);
                        }
                    }
                    Err(e) => {
                        // Treat it as single-off event
                        eprintln!(
                            "Failed to parse recurrence for event {:?}: {}",
                            event.get_summary(),
                            e
                        );
                        if !is_past_event(event) {
                            events.push(EventItem::from(event));
                        } else {
                            past_events.push(EventItem::from(event));
                        }
                    }
                }
            } else if is_past_event(event) {
                past_events.push(EventItem::from(event));
            } else {
                events.push(EventItem::from(event));
            }
        });

        let (mut completed_tasks, remaining): (Vec<TaskItem>, Vec<TaskItem>) =
            cal.todos().map(TaskItem::new).partition(|t| t.completed);
        let (mut upcoming_tasks, mut tasks): (Vec<TaskItem>, Vec<TaskItem>) =
            remaining.into_iter().partition(|t| {
                t.start.as_ref().is_some_and(|s| {
                    let today = Local::now().date_naive();
                    match s {
                        DatePerhapsTime::Date(d) => *d > today,
                        DatePerhapsTime::DateTime(cdt) => get_naive_date(cdt) > today,
                    }
                })
            });

        completed_tasks.sort();
        upcoming_tasks.sort();
        tasks.sort();
        events.sort();
        recurring_events.sort();
        past_events.sort();

        Self {
            events,
            recurring_events,
            past_events,
            tasks,
            completed_tasks,
            upcoming_tasks,
        }
    }

    pub fn active_events(&self) -> Vec<EventItem> {
        self.events
            .iter()
            .chain(&self.recurring_events)
            .cloned()
            .collect()
    }

    pub fn tasks(&self) -> Vec<TaskItem> {
        self.tasks.clone()
    }
}

#[derive(Clone)]
pub struct State {
    cal: HashMap<String, MiniCal>,
    readonly: bool,
}

pub enum FailReason {
    NotAllowed,
    CannotWrite,
}

impl State {
    pub fn new(
        dir: PathBuf,
        readonly: bool,
        max_recurrence_count: u32,
        max_recurrence_date: u32,
    ) -> Self {
        fn load_calendar(
            path: PathBuf,
            max_recurrence_count: u32,
            max_recurrence_date: u32,
        ) -> MiniCal {
            let mut cal = Calendar::new();

            if let Ok(entries) = fs::read_dir(&path) {
                entries.filter_map(|e| e.ok()).for_each(|e| {
                    if e.path().extension().and_then(|e| e.to_str()) == Some("ics") {
                        if let Ok(contents) = fs::read_to_string(e.path()) {
                            if let Ok(parsed) = Calendar::from_str(&contents) {
                                cal.extend(parsed.components);
                            } else {
                                eprintln!("Failed to parse {:?}", e.path());
                            }
                        } else {
                            eprintln!("Failed to read from file {:?}", e.path());
                        }
                    }
                });
            } else {
                eprintln!("Failed to list files in {:?}", path);
            }
            debug!("Loaded {} components from {:?}", cal.components.len(), path);
            MiniCal::from_calendar(&cal, max_recurrence_count, max_recurrence_date)
        }

        let cals: Vec<DirEntry> = fs::read_dir(&dir)
            .unwrap_or_else(|e| {
                panic!(
                    "Warning: Failed to read calendar directory {}: {}",
                    dir.display(),
                    e
                );
            })
            .filter_map(|r| r.ok())
            .filter(|e| {
                if let Ok(t) = e.file_type() {
                    t.is_dir()
                } else {
                    false
                }
            })
            .collect();

        if cals.len() == 0 {
            panic!(
                "No calendars discovered. There needs to be at least one directory inside {:?} that contains calendar items (.ics).",
                dir
            );
        }

        debug!("Discovered {} calendars.", cals.len());
        cals.iter()
            .for_each(|c| debug!("Calendar {:?} found in path {:?}", c.file_name(), c.path()));

        let cal: HashMap<String, MiniCal> = cals
            .into_iter()
            .map(|c| {
                (
                    c.file_name().to_string_lossy().to_string(),
                    load_calendar(c.path(), max_recurrence_count, max_recurrence_date),
                )
            })
            .collect();

        Self { cal, readonly }
    }

    pub fn calendar_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.cal.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn get_events(&self, cal: Option<&str>) -> Vec<EventItem> {
        let get_all = || {
            let mut events: Vec<EventItem> =
                self.cal.values().flat_map(|c| c.active_events()).collect();
            events.sort();
            events
        };
        if let Some(name) = cal {
            if let Some(cal) = self.cal.get(name) {
                cal.active_events()
            } else {
                get_all()
            }
        } else {
            get_all()
        }
    }

    pub fn get_tasks(&self, cal: Option<&str>) -> Vec<TaskItem> {
        let get_all = || {
            let mut tasks: Vec<TaskItem> = self.cal.values().flat_map(|c| c.tasks()).collect();
            tasks.sort();
            tasks
        };
        if let Some(name) = cal {
            if let Some(cal) = self.cal.get(name) {
                cal.tasks()
            } else {
                get_all()
            }
        } else {
            get_all()
        }
    }
}
