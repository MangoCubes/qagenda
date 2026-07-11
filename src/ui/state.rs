use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tab {
    /// Displays tasks only. Has a section below that contains all completed tasks.
    Tasks {
        past: bool,
        /// If set to None, display all events
        /// If set to a value, then display all events from that calendar only
        cal: Option<String>,
    },
    /// Displays tasks and events. Tasks are displayed only if [`show_tasks`] is true and the due
    /// date is the curretly displayed date.
    Events {
        show_tasks: bool,
        /// If set to None, display all events
        /// If set to a value, then display all events from that calendar only
        cal: Option<String>,
    },
}

impl Default for Tab {
    fn default() -> Self {
        Self::Events {
            show_tasks: false,
            cal: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DisplayType {
    Day,
    Week,
    #[default]
    Month,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Focus {
    #[default]
    Agenda,
    Calendar,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct UIState {
    pub tab: Tab,
    pub cal: DisplayType,
    pub focus: Focus,
}

impl UIState {
    pub fn selected_cal(&self) -> Option<String> {
        match &self.tab {
            Tab::Tasks { cal, .. } => cal.clone(),
            Tab::Events { cal, .. } => cal.clone(),
        }
    }

    pub fn set_selected_cal(&mut self, cal: Option<String>) {
        match &mut self.tab {
            Tab::Tasks { cal: c, .. } => *c = cal,
            Tab::Events { cal: c, .. } => *c = cal,
        }
    }
}
