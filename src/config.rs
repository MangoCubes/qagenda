pub mod io;
pub mod keybinds;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{config::keybinds::KeyBinds, ui::state::UIState};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum Anchor {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub dir: PathBuf,
    pub expand: bool,
    pub anchor: Anchor,
    pub css: String,
    #[serde(rename = "initState")]
    pub init_state: UIState,
    pub keybinds: KeyBinds,
    /// Maximum number of recurrences to display for recurring events.
    /// Defaults to 3. Values below 1 are invalid and will panic.
    #[serde(rename = "maxRecurrenceCount")]
    pub max_recurrence_count: u32,
    /// Maximum number of days into the future for which recurring events should appear.
    /// Set to 0 to disable this limit (default).
    #[serde(rename = "maxRecurrenceDate")]
    pub max_recurrence_date: u32,
    /// If true, then you can click on other windows even when the widget is present. False is
    /// recommended for those whose workflow is heavily keyboard-oriented, as you can reliably close
    /// the widget with the quit key. If this is set to true and you accidentally focused something
    /// else, then you would have to focus the widget before closing it, either by clicking on it or
    /// hovering your mouse over it.
    #[serde(rename = "allowUnfocused")]
    pub allow_unfocused: bool,
}

impl Default for Config {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("events");
        #[cfg(not(debug_assertions))]
        let dir = PathBuf::from(std::env::var("HOME").expect("No home???")).join(".calendar");
        Self {
            dir,
            expand: true,
            anchor: Anchor::TopRight,
            css: include_str!("./default.css").to_string(),
            init_state: UIState::default(),
            keybinds: KeyBinds::default(),
            max_recurrence_count: 3,
            max_recurrence_date: 30,
            allow_unfocused: false,
        }
    }
}

impl Config {
    pub fn validate(&self) {
        assert!(
            self.max_recurrence_count >= 1,
            "maxRecurrenceCount must be at least 1, got {}",
            self.max_recurrence_count
        );
    }

    pub fn get_edges(&self) -> (bool, bool, bool, bool) {
        let expand = self.expand;
        match self.anchor {
            Anchor::Top => (true, false, expand, expand),
            Anchor::Bottom => (false, true, expand, expand),
            Anchor::Left => (expand, expand, true, false),
            Anchor::Right => (expand, expand, false, true),
            Anchor::TopLeft => (true, false, true, false),
            Anchor::TopRight => (true, false, false, true),
            Anchor::BottomLeft => (false, true, true, false),
            Anchor::BottomRight => (false, true, false, true),
            Anchor::Center => (expand, expand, expand, expand),
        }
    }
}
