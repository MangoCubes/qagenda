use std::collections::HashMap;

use gtk4::gdk::{Key, ModifierType};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinds(HashMap<(Key, ModifierType), Action>);

impl Default for KeyBinds {
    fn default() -> Self {
        Self(HashMap::from([
            ((Key::Up, ModifierType::empty()), Action::Up),
            ((Key::Down, ModifierType::empty()), Action::Down),
            ((Key::Left, ModifierType::empty()), Action::Left),
            ((Key::Right, ModifierType::empty()), Action::Right),
            ((Key::Up, ModifierType::CONTROL_MASK), Action::SectionUp),
            ((Key::Down, ModifierType::CONTROL_MASK), Action::SectionDown),
        ]))
    }
}

impl KeyBinds {
    pub fn get(&self, key: &Key, mods: ModifierType) -> Option<&Action> {
        self.0.get(&(*key, mods))
    }
}

impl Serialize for KeyBinds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
            .iter()
            .map(|((k, m), v)| (gtk4::accelerator_name(*k, *m).to_string(), v))
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
                if let Some((key, mods)) = gtk4::accelerator_parse(&k) {
                    Some(((key, mods), v))
                } else {
                    eprintln!("Failed to parse key \"{}\"; Skipping.", k);
                    None
                }
            })
            .collect();
        Ok(KeyBinds(keybinds))
    }
}
