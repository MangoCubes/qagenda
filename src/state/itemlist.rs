use std::vec::IntoIter;

use crate::state::{event::EventItem, task::TaskItem};

pub type TaskList = ItemList<TaskItem>;
pub type EventList = ItemList<EventItem>;

pub enum ItemList<T> {
    Mixed(Vec<(String, T)>),
    Single((String, Vec<T>)),
}

impl<T> ItemList<T> {
    pub fn is_empty(&self) -> bool {
        match self {
            ItemList::Mixed(v) => v.is_empty(),
            ItemList::Single((_, v)) => v.is_empty(),
        }
    }

    pub fn get(&self, current_item: usize) -> Option<(&String, &T)> {
        match self {
            ItemList::Mixed(v) => v.get(current_item).map(|(n, t)| (n, t)),
            ItemList::Single((n, v)) => v.get(current_item).map(|t| (n, t)),
        }
    }
}

impl<T> IntoIterator for ItemList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ItemList::Mixed(v) => v
                .into_iter()
                .map(|(_, item)| item)
                .collect::<Vec<T>>()
                .into_iter(),
            ItemList::Single((_, v)) => v.into_iter(),
        }
    }
}

impl<T> ItemList<T> {
    pub fn into_named_iter(self) -> IntoIter<(String, T)> {
        match self {
            ItemList::Mixed(v) => v.into_iter(),
            ItemList::Single((name, v)) => v
                .into_iter()
                .map(move |item| (name.clone(), item))
                .collect::<Vec<(String, T)>>()
                .into_iter(),
        }
    }
}
