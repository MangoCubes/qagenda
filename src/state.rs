use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use icalendar::Calendar;

use crate::{debug, logging::is_verbose};

#[derive(Clone)]
pub struct State {
    cal: Vec<Calendar>,
    readonly: bool,
}

pub enum FailReason {
    NotAllowed,
    CannotWrite,
}

impl State {
    pub fn new(dir: PathBuf, readonly: bool) -> Self {
        fn load_calendar(path: PathBuf) -> Calendar {
            use std::str::FromStr;
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
                })
            } else {
                eprintln!("Failed to list files in {:?}", path);
            }
            debug!("Loaded {} components from {:?}", cal.components.len(), path);
            cal
        }

        let Ok(entries) = fs::read_dir(&dir) else {
            panic!("Failed to read directory: {}", dir.display());
        };
        let cals: Vec<DirEntry> = entries
            .filter_map(|r| r.ok())
            .filter(|e| {
                if let Ok(t) = e.file_type() {
                    t.is_dir()
                } else {
                    false
                }
            })
            .collect();
        debug!("Discovered {} calendars.", cals.len());

        if is_verbose() {
            cals.iter()
                .for_each(|c| debug!("Calendar {:?} found in path {:?}", c.file_name(), c.path()));
        }

        Self {
            cal: cals.into_iter().map(|c| load_calendar(c.path())).collect(),
            readonly,
        }
    }

    pub fn toggle_task_complete(&self) -> Result<(), FailReason> {
        if self.readonly {
            return Err(FailReason::NotAllowed);
        }
        Ok(())
    }
}
