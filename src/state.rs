pub mod event;
pub mod task;
pub mod utils;

use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::PathBuf,
    str::FromStr,
};

use icalendar::Calendar;

use crate::{
    debug,
    state::{event::EventItem, task::TaskItem},
};

#[derive(Clone)]
pub struct State {
    cal: HashMap<String, Calendar>,
    readonly: bool,
}

pub enum FailReason {
    NotAllowed,
    CannotWrite,
}

impl State {
    pub fn new(dir: PathBuf, readonly: bool) -> Self {
        fn load_calendar(path: PathBuf) -> Calendar {
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
            cal
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

        let cal: HashMap<String, Calendar> = cals
            .into_iter()
            .map(|c| {
                let name = c.file_name().to_string_lossy().to_string();
                (name, load_calendar(c.path()))
            })
            .collect();

        Self { cal, readonly }
    }

    pub fn calendar_names(&self) -> Vec<String> {
        self.cal.keys().cloned().collect()
    }

    pub fn get_events(&self, cal: Option<&str>) -> Vec<EventItem> {
        let cals: Vec<&Calendar> = match cal {
            Some(name) => self.cal.get(name).into_iter().collect(),
            None => self.cal.values().collect(),
        };

        cals.iter()
            .map(|c| {
                c.events()
                    .map(|e| EventItem::new(e))
                    .collect::<Vec<EventItem>>()
            })
            .flatten()
            .collect()
    }

    pub fn get_tasks(&self, cal_filter: Option<&str>) -> Vec<TaskItem> {
        let cals: Vec<&Calendar> = match cal_filter {
            Some(name) => self.cal.get(name).into_iter().collect(),
            None => self.cal.values().collect(),
        };

        cals.iter()
            .map(|c| {
                c.todos()
                    .map(|e| TaskItem::new(e))
                    .collect::<Vec<TaskItem>>()
            })
            .flatten()
            .collect()
    }
}
