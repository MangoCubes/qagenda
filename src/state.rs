pub mod details;
pub mod diff;
pub mod event;
pub mod minical;
pub mod task;
pub mod utils;

use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::PathBuf,
    str::FromStr,
};

use icalendar::{Calendar, CalendarComponent, Component};

use crate::{
    debug, error,
    state::{diff::Diff, event::EventItem, minical::MiniCal, task::TaskItem},
    types::{CalPath, CalsPath},
};

#[derive(Clone)]
pub struct State {
    cal: HashMap<String, MiniCal>,
    dir: CalsPath,
    dry_run: bool,
    pub pending: Diff,
}

impl State {
    pub fn new(
        dir: CalsPath,
        dry_run: bool,
        max_recurrence_count: u32,
        max_recurrence_date: u32,
    ) -> Self {
        fn load_calendar(
            name: &String,
            path: CalPath,
            max_recurrence_count: u32,
            max_recurrence_date: u32,
        ) -> Result<MiniCal, String> {
            if let Ok(entries) = fs::read_dir(&path.0) {
                let comps: HashMap<PathBuf, Vec<CalendarComponent>> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|e| e.to_str()) == Some("ics"))
                    .filter_map(|e| {
                        let p = e.path();
                        if let Ok(contents) = fs::read_to_string(&p) {
                            if let Ok(parsed) = Calendar::from_str(&contents) {
                                Some((p, parsed.components))
                            } else {
                                eprintln!("Failed to parse {:?}", p);
                                None
                            }
                        } else {
                            eprintln!("Failed to read from file {:?}", p);
                            None
                        }
                    })
                    .collect();
                debug!("Loaded {} components from {:?}", comps.len(), path.0);
                Ok(MiniCal::from_calendar(
                    name,
                    comps,
                    max_recurrence_count,
                    max_recurrence_date,
                ))
            } else {
                Err(format!("Failed to list files in {:?}", path.0))
            }
        }

        let cals: Vec<DirEntry> = fs::read_dir(&dir.0)
            .unwrap_or_else(|e| {
                panic!(
                    "Warning: Failed to read calendar directory {}: {}",
                    dir.0.to_string_lossy(),
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
            .filter_map(|c| {
                let name = c.file_name().to_string_lossy().to_string();
                match load_calendar(
                    &name,
                    CalPath(c.path()),
                    max_recurrence_count,
                    max_recurrence_date,
                ) {
                    Ok(c) => Some((name.clone(), c)),
                    Err(err) => {
                        error!("Failed to load calendar: {}", err);
                        None
                    }
                }
            })
            .collect();

        Self {
            cal,
            dir,
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
        (match cal {
            Some(name) => self.cal[name].incomplete_tasks_count(),
            None => self
                .cal
                .iter()
                .fold(0, |count, (_, c)| count + c.incomplete_tasks_count()),
        }) + self.pending.new_tasks_count(cal)
    }
    pub fn get_events_count(&self, cal: Option<&str>) -> usize {
        (match cal {
            Some(name) => self.cal[name].active_events_count(),
            None => self
                .cal
                .iter()
                .fold(0, |count, (_, c)| count + c.active_events_count()),
        }) + self.pending.new_events_count(cal)
    }

    pub fn get_events(&self, cal: Option<&str>) -> Vec<EventItem> {
        let mut events: Vec<EventItem> = cal.map_or_else(
            || {
                self.cal
                    .values()
                    .flat_map(|c| c.active_events())
                    .cloned()
                    .collect()
            },
            |name| {
                self.cal.get(name).map_or_else(Vec::new, |c| {
                    c.active_events().into_iter().cloned().collect()
                })
            },
        );
        events.extend(self.pending.get_new_events(cal));
        events.sort_unstable();
        events
    }

    pub fn get_tasks(&self, cal: Option<&str>) -> Vec<TaskItem> {
        let mut tasks: Vec<TaskItem> = cal.map_or_else(
            || {
                self.cal
                    .values()
                    .flat_map(|c| c.incomplete_tasks())
                    .cloned()
                    .collect()
            },
            |name| {
                self.cal.get(name).map_or_else(Vec::new, |c| {
                    c.incomplete_tasks().into_iter().cloned().collect()
                })
            },
        );
        tasks.extend(self.pending.get_new_tasks(cal));
        tasks.sort_unstable();
        tasks
    }

    pub fn write_to_disk(&self) -> Result<(), String> {
        if self.dry_run {
            return Ok(());
        }

        self.cal.keys().for_each(|cal| {
            let diff = self.pending.get_cal_diff(cal);

            diff.new_events.into_iter().for_each(|event| {
                let mut cal = Calendar::new();
                cal.push(event.to_event());
                if let Err(e) = fs::write(&event.path.0, cal.to_string()) {
                    error!("Failed to write new event file {:?}: {}", event.path.0, e);
                }
            });

            diff.new_tasks.into_iter().for_each(|task| {
                let mut cal = Calendar::new();
                cal.push(task.to_todo());
                if let Err(e) = fs::write(&task.path.0, cal.to_string()) {
                    error!("Failed to write new task file {:?}: {}", task.path.0, e);
                }
            });

            diff.events.into_iter().for_each(|(uuid, (_, new))| {
                if let Err(e) = fs::read_to_string(&new.path.0)
                    .map_err(|e| e.to_string())
                    .and_then(|p| Calendar::from_str(&p))
                    .and_then(|mut cal| {
                        let event = cal
                            .components
                            .iter_mut()
                            .find_map(|ev| match ev {
                                CalendarComponent::Event(e)
                                    if e.get_uid() == Some(uuid.as_str()) =>
                                {
                                    Some(e)
                                }
                                _ => None,
                            })
                            .ok_or("Failed to locate the event within the file!")?;
                        new.write_to(event);
                        fs::write(&new.path.0, cal.to_string()).map_err(|e| e.to_string())
                    })
                {
                    error!("Failed to update the calendar event: {}", e);
                };
            });

            diff.tasks.into_iter().for_each(|(uuid, (_, new))| {
                if let Err(e) = fs::read_to_string(&new.path.0)
                    .map_err(|e| e.to_string())
                    .and_then(|p| Calendar::from_str(&p))
                    .and_then(|mut cal| {
                        let task = cal
                            .components
                            .iter_mut()
                            .find_map(|ev| match ev {
                                CalendarComponent::Todo(e)
                                    if e.get_uid() == Some(uuid.as_str()) =>
                                {
                                    Some(e)
                                }
                                _ => None,
                            })
                            .ok_or("Failed to locate the task within the file!")?;
                        new.write_to(task);
                        fs::write(&new.path.0, cal.to_string()).map_err(|e| e.to_string())
                    })
                {
                    error!("Failed to update the calendar task: {}", e);
                };
            });

            // Deleted items may be part of an ics file that contains multiple events/tasks so we
            // can't just delete the corresponding files

            diff.deleted_events.into_iter().for_each(|(uuid, path)| {
                if let Err(e) = fs::read_to_string(&path.0)
                    .map_err(|e| e.to_string())
                    .and_then(|p| Calendar::from_str(&p))
                    .and_then(|mut cal| {
                        cal.components.retain(|ev| match ev {
                            CalendarComponent::Event(e) => e.get_uid() != Some(uuid.as_str()),
                            _ => true,
                        });
                        if cal.components.is_empty() {
                            fs::remove_file(&path.0).map_err(|e| e.to_string())
                        } else {
                            fs::write(&path.0, cal.to_string()).map_err(|e| e.to_string())
                        }
                    })
                {
                    error!("Failed to delete calendar event: {}", e);
                };
            });

            diff.deleted_tasks.into_iter().for_each(|(uuid, path)| {
                if let Err(e) = fs::read_to_string(&path.0)
                    .map_err(|e| e.to_string())
                    .and_then(|p| Calendar::from_str(&p))
                    .and_then(|mut cal| {
                        cal.components.retain(|ev| match ev {
                            CalendarComponent::Todo(e) => e.get_uid() != Some(uuid.as_str()),
                            _ => true,
                        });
                        if cal.components.is_empty() {
                            fs::remove_file(&path.0).map_err(|e| e.to_string())
                        } else {
                            fs::write(&path.0, cal.to_string()).map_err(|e| e.to_string())
                        }
                    })
                {
                    error!("Failed to delete the calendar task: {}", e);
                };
            });
        });

        Ok(())
    }
}
