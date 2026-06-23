use std::{
    fs::{self, DirEntry},
    path::PathBuf,
};

use icalendar::Calendar;

use crate::debug;

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
        Self {
            cal: todo!(),
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
