pub mod io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ui::UIState;

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
        }
    }
}

impl Config {
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
