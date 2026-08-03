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

/// Represents differences found in a single calendar task or event. Contains natural language
/// description of the changes.
pub enum SingleDiff {
    Create {
        summary: String,
    },
    Delete {
        summary: String,
    },
    Update {
        summary: String,
        changes: Vec<String>,
    },
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

    fn prepare_update_task(&mut self, task: &TaskItem) -> &mut TaskItem {
        &mut self
            .tasks
            .entry(task.cal.clone())
            .or_insert(HashMap::new())
            .entry(task.uid.clone())
            .or_insert((task.clone(), task.clone()))
            .1
    }

    fn toggle_task(&mut self, task: &TaskItem) {
        let r = self.prepare_update_task(task);
        r.completed = !r.completed;
    }

    fn prepare_update_event(&mut self, event: &EventItem) -> &mut EventItem {
        &mut self
            .events
            .entry(event.cal.clone())
            .or_insert(HashMap::new())
            .entry(event.uid.clone())
            .or_insert((event.clone(), event.clone()))
            .1
    }

    fn get_changes(&self) -> HashMap<String, Vec<SingleDiff>> {
        let mut cal_changes: HashMap<String, Vec<SingleDiff>> = HashMap::new();
        self.new_events.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|e| SingleDiff::Create {
                    summary: format!("Create new event \"{}\"", e.summary),
                })
                .collect::<Vec<SingleDiff>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.new_tasks.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|e| SingleDiff::Create {
                    summary: format!("Create new task \"{}\"", e.summary),
                })
                .collect::<Vec<SingleDiff>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.events.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|(_, (old, new))| old.diff(new))
                .collect::<Vec<SingleDiff>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.tasks.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|(_, (old, new))| old.diff(new))
                .collect::<Vec<SingleDiff>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.deleted_events.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|(_, s)| SingleDiff::Delete {
                    summary: format!("Delete event \"{}\"", s),
                })
                .collect::<Vec<SingleDiff>>();

            cal_changes.entry(c.clone()).or_default().extend(msgs);
        });
        self.deleted_tasks.iter().for_each(|(c, es)| {
            let msgs = es
                .iter()
                .map(|(_, s)| SingleDiff::Delete {
                    summary: format!("Delete task \"{}\"", s),
                })
                .collect::<Vec<SingleDiff>>();

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

    pub fn toggle_task(&self, task: &TaskItem) {
        self.inner.write().unwrap().toggle_task(task);
    }

    pub fn get_changes(&self) -> HashMap<String, Vec<SingleDiff>> {
        self.inner.read().unwrap().get_changes()
    }

    pub fn has_changes(&self) -> bool {
        let mut inner = self.inner.write().unwrap();

        inner.events.values_mut().for_each(|cal| {
            cal.retain(|_, (old, new)| old != new);
        });
        inner.events.retain(|_, cal| !cal.is_empty());

        inner.tasks.values_mut().for_each(|cal| {
            cal.retain(|_, (old, new)| old != new);
        });
        inner.tasks.retain(|_, cal| !cal.is_empty());

        !inner.new_events.is_empty()
            || !inner.new_tasks.is_empty()
            || !inner.events.is_empty()
            || !inner.tasks.is_empty()
            || !inner.deleted_events.is_empty()
            || !inner.deleted_tasks.is_empty()
    }

    pub fn is_task_completed(&self, task: &TaskItem) -> bool {
        self.inner
            .read()
            .unwrap()
            .tasks
            .get(&task.cal)
            .and_then(|cal_tasks| cal_tasks.get(&task.uid))
            .map(|(_, modified)| modified.completed)
            .unwrap_or(task.completed)
    }
}
