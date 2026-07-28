use std::collections::HashMap;

use crate::{
    state::{event::EventItem, task::TaskItem},
    types::UUID,
};

#[derive(Debug, Clone)]
pub struct Diff {
    events: HashMap<String, HashMap<UUID, EventItem>>,
    tasks: HashMap<String, HashMap<UUID, TaskItem>>,
}

impl Diff {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            tasks: HashMap::new(),
        }
    }
}
