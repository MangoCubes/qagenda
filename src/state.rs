pub mod details;
pub mod diff;
pub mod event;
pub mod itemlist;
pub mod minical;
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
    state::{
        diff::Diff,
        event::EventItem,
        itemlist::{EventList, TaskList},
        minical::MiniCal,
        task::TaskItem,
    },
};

#[derive(Clone)]
pub struct State {
    cal: HashMap<String, MiniCal>,
    dry_run: bool,
    pending: Diff,
}

impl State {
    pub fn new(
        dir: PathBuf,
        dry_run: bool,
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

        Self {
            cal,
            dry_run,
            pending: Diff::new(),
        }
    }

    pub fn calendar_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.cal.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn get_tasks_count(&self, cal: Option<&str>) -> usize {
        match cal {
            Some(name) => self.cal[name].incomplete_tasks_count(),
            None => self
                .cal
                .iter()
                .fold(0, |count, (_, c)| count + c.incomplete_tasks_count()),
        }
    }
    pub fn get_events_count(&self, cal: Option<&str>) -> usize {
        match cal {
            Some(name) => self.cal[name].active_events_count(),
            None => self
                .cal
                .iter()
                .fold(0, |count, (_, c)| count + c.active_events_count()),
        }
    }

    pub fn get_events(&self, cal: Option<&str>) -> EventList {
        match cal {
            Some(name) => EventList::Single((name.to_string(), self.cal[name].active_events())),
            None => {
                let mut events: Vec<(String, EventItem)> = self
                    .cal
                    .iter()
                    .flat_map(|(name, c)| c.active_events().into_iter().map(|e| (name.clone(), e)))
                    .collect();
                events.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
                EventList::Mixed(events)
            }
        }
    }

    pub fn get_tasks(&self, cal: Option<&str>) -> TaskList {
        match cal {
            Some(name) => TaskList::Single((name.to_string(), self.cal[name].incomplete_tasks())),
            None => {
                let mut tasks: Vec<(String, TaskItem)> = self
                    .cal
                    .iter()
                    .flat_map(|(name, c)| {
                        c.incomplete_tasks().into_iter().map(|e| (name.clone(), e))
                    })
                    .collect();
                tasks.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
                TaskList::Mixed(tasks)
            }
        }
    }
}
