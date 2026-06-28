use std::collections::HashMap;

use gtk4::gdk::Key;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    SectionUp,
    SectionDown,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct KeyBinds(HashMap<Key, Action>);

impl Serialize for KeyBinds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
            .iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<HashMap<String, &Action>>()
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for KeyBinds {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let keybinds = HashMap::<String, Action>::deserialize(deserializer)?
            .into_iter()
            .filter_map(|(k, v)| {
                if let Some(key) = Key::from_name(&k) {
                    Some((key, v))
                } else {
                    eprintln!("Unknown key {} found; Skipping.", k);
                    None
                }
            })
            .collect();
        Ok(KeyBinds(keybinds))
    }
}
