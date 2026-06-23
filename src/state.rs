use std::path::PathBuf;

use icalendar::Calendar;

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
