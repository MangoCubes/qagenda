use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UUID = String;

/// Path to calendars, must be directory
/// The program expects this directory to contain directories, each corresponding to a calendar
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CalsPath(pub PathBuf);

/// Path to a single calendaar, must be directory
/// The program expects this directory to contain ics files
pub struct CalPath(pub PathBuf);

/// Path to calendar items (task/event), must be a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPath(pub PathBuf);

impl CalsPath {
    pub fn to_itempath(&self, cal: &str, name: &str) -> ItemPath {
        ItemPath(self.0.join(&cal).join(format!("{}.ics", name)))
    }
    pub fn new_item(&self, cal: &str) -> (ItemPath, UUID) {
        let uid = Uuid::new_v4().to_string();
        (
            ItemPath(self.0.join(&cal).join(format!("{}.ics", uid))),
            uid,
        )
    }
    pub fn to_calpath(&self, cal: &str) -> CalPath {
        CalPath(self.0.join(&cal))
    }
}
