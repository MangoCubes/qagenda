use std::sync::{Arc, RwLock};

use chrono::{Datelike, Local};
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
struct InnerUIState {
    tab: Tab,
    cal: DisplayType,
    focus: Focus,
    year: i32,
    month: u32,
}

impl Default for InnerUIState {
    fn default() -> Self {
        let now = Local::now().date_naive();
        Self {
            tab: Tab::default(),
            cal: DisplayType::default(),
            focus: Focus::default(),
            year: now.year(),
            month: now.month(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UIState {
    inner: Arc<RwLock<InnerUIState>>,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerUIState::default())),
        }
    }
}

impl Serialize for UIState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.read().unwrap().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UIState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            inner: Arc::new(RwLock::new(InnerUIState::deserialize(deserializer)?)),
        })
    }
}

impl UIState {
    pub fn focus(&self) -> Focus {
        self.inner.read().unwrap().focus
    }

    pub fn set_focus(&self, focus: Focus) {
        self.inner.write().unwrap().focus = focus;
    }

    pub fn tab(&self) -> Tab {
        self.inner.read().unwrap().tab.clone()
    }

    pub fn toggle_tab(&self) {
        let mut guard = self.inner.write().unwrap();
        let cal = match &guard.tab {
            Tab::Tasks { cal, .. } => cal.clone(),
            Tab::Events { cal, .. } => cal.clone(),
        };
        guard.tab = match &guard.tab {
            Tab::Events { .. } => Tab::Tasks { past: false, cal },
            Tab::Tasks { .. } => Tab::Events {
                show_tasks: false,
                cal,
            },
        };
    }

    pub fn year(&self) -> i32 {
        self.inner.read().unwrap().year
    }

    pub fn month(&self) -> u32 {
        self.inner.read().unwrap().month
    }

    pub fn cycle_month(&self, next: bool) {
        let mut guard = self.inner.write().unwrap();
        if next {
            if guard.month == 12 {
                guard.month = 1;
                guard.year += 1;
            } else {
                guard.month += 1;
            }
        } else if guard.month == 1 {
            guard.month = 12;
            guard.year -= 1;
        } else {
            guard.month -= 1;
        }
    }

    pub fn selected_cal(&self) -> Option<String> {
        let guard = self.inner.read().unwrap();
        match &guard.tab {
            Tab::Tasks { cal, .. } => cal.clone(),
            Tab::Events { cal, .. } => cal.clone(),
        }
    }

    pub fn set_selected_cal(&self, cal: Option<String>) {
        let mut guard = self.inner.write().unwrap();
        match &mut guard.tab {
            Tab::Tasks { cal: c, .. } => *c = cal,
            Tab::Events { cal: c, .. } => *c = cal,
        }
    }

    pub fn reset_month(&self) {
        let now = Local::now().date_naive();
        let mut guard = self.inner.write().unwrap();
        guard.year = now.year();
        guard.month = now.month();
    }
}
