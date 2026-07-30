use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{
    state::{event::EventItem, task::TaskItem},
    types::UUID,
};

#[derive(Debug, Clone)]
struct InnerDiff {
    new_events: HashMap<String, Vec<EventItem>>,
    new_tasks: HashMap<String, Vec<TaskItem>>,
    /// Contains pairs of original and new event item
    events: HashMap<String, HashMap<UUID, (EventItem, EventItem)>>,
    tasks: HashMap<String, HashMap<UUID, (TaskItem, TaskItem)>>,
    /// Contains UUID of the event to delete, and its corresponding summary
    deleted_events: HashMap<String, Vec<(UUID, String)>>,
    deleted_tasks: HashMap<String, Vec<(UUID, String)>>,
}

impl InnerDiff {
    fn new() -> Self {
        Self {
            new_events: HashMap::new(),
            new_tasks: HashMap::new(),
            events: HashMap::new(),
            tasks: HashMap::new(),
            deleted_events: HashMap::new(),
            deleted_tasks: HashMap::new(),
        }
    }
    fn prepare_update_task(&mut self, cal: &String, task: &TaskItem) -> &mut TaskItem {
        &mut self
            .tasks
            .entry(cal.clone())
            .or_insert(HashMap::new())
            .entry(task.uid.clone())
            .or_insert((task.clone(), task.clone()))
            .1
    }

    fn toggle_task(&mut self, cal: &String, task: &TaskItem) {
        let r = self.prepare_update_task(cal, task);
        r.completed = !r.completed;
    }

    fn get_changes(&self) -> HashMap<String, Vec<String>> {
        let mut cal_changes: HashMap<String, Vec<String>> = HashMap::new();
        self.new_events.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|e| format!("Create new event \"{}\"", e.summary))
                .collect::<Vec<String>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.new_tasks.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|e| format!("Create new task \"{}\"", e.summary))
                .collect::<Vec<String>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.events.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|(_, (old, new))| old.diff(new))
                .flatten()
                .collect::<Vec<String>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.tasks.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|(_, (old, new))| old.diff(new))
                .flatten()
                .collect::<Vec<String>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.deleted_events.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|(_, s)| format!("Delete event \"{}\"", s))
                .collect::<Vec<String>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.deleted_tasks.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|(_, s)| format!("Delete task \"{}\"", s))
                .collect::<Vec<String>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        cal_changes
    }
}

#[derive(Debug, Clone)]
pub struct Diff {
    inner: Arc<RwLock<InnerDiff>>,
}

impl Diff {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerDiff::new())),
        }
    }

    pub fn toggle_task(&self, cal: &String, task: &TaskItem) {
        self.inner.write().unwrap().toggle_task(cal, task);
    }

    pub fn get_changes(&self) -> HashMap<String, Vec<String>> {
        self.inner.read().unwrap().get_changes()
    }
}
