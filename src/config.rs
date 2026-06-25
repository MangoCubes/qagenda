mod io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("events");
        #[cfg(not(debug_assertions))]
        let dir = PathBuf::from(std::env::var("HOME").expect("No home???")).join(".calendar");
        Self { dir }
    }
}
