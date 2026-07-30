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

    pub fn get(&self, current_item: usize) -> &T {
        match self {
            ItemList::Mixed(v) => &v[current_item].1,
            ItemList::Single(v) => &v.1[current_item],
        }
    }
}

pub enum ItemListIter<T> {
    Mixed(IntoIter<T>),
    Single(IntoIter<T>),
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
