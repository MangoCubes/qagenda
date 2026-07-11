use std::{
    iter,
    sync::{Arc, Mutex},
};

use gtk4::{
    Align, Box, Label, Orientation,
    prelude::{BoxExt, WidgetExt},
};

use crate::{
    state::State,
    ui::state::{Tab, UIState},
};

#[derive(Clone)]
pub struct Widget {
    pub agenda: Box,
    pub title: Label,
    pub cal_indicator: Box,
    pub ui_state: Arc<Mutex<UIState>>,
    pub state: Arc<Mutex<State>>,
}

impl Widget {
    pub fn new(ui_state: Arc<Mutex<UIState>>, state: Arc<Mutex<State>>) -> Self {
        let title = Label::new(None);
        title.set_halign(Align::Start);
        title.add_css_class("section-title");

        let cal_indicator = Box::new(Orientation::Horizontal, 4);
        cal_indicator.set_halign(Align::Fill);
        cal_indicator.set_hexpand(true);
        cal_indicator.set_homogeneous(true);

        let agenda = Box::new(Orientation::Vertical, 4);

        let widget = Self {
            agenda,
            title,
            cal_indicator,
            ui_state,
            state,
        };

        widget.init_indicators();
        widget.update();
        widget
    }

    fn init_indicators(&self) {
        let st = self.state.lock().unwrap();
        st.calendar_names().iter().for_each(|_| {
            let box_item = Box::new(Orientation::Horizontal, 0);
            box_item.set_size_request(-1, -1);
            box_item.set_halign(Align::Fill);
            box_item.set_hexpand(true);
            box_item.add_css_class("cal-indicator");
            self.cal_indicator.append(&box_item);
        });
    }

    pub fn update(&self) {
        let ui = self.ui_state.lock().unwrap();
        let state = self.state.lock().unwrap();

        let tab_name = match ui.tab {
            Tab::Events { .. } => "Events",
            Tab::Tasks { .. } => "Tasks",
        };
        let title_text = match ui.selected_cal().as_deref() {
            Some(name) => format!("{} - {}", tab_name, name),
            None => format!("{} (All calendars)", tab_name),
        };
        self.title.set_text(&title_text);

        let selected = ui.selected_cal();
        let show_all = selected.is_none();

        state
            .calendar_names()
            .iter()
            .zip(iter::successors(self.cal_indicator.first_child(), |w| {
                w.next_sibling()
            }))
            .for_each(|(name, widget)| {
                if show_all || selected == Some(name.to_string()) {
                    widget.add_css_class("cal-indicator-active");
                } else {
                    widget.remove_css_class("cal-indicator-active");
                }
            });

        while let Some(child) = self.agenda.first_child() {
            self.agenda.remove(&child);
        }

        match &ui.tab {
            Tab::Events { cal, .. } => {
                let events = state.get_events(cal.as_deref());
                if events.is_empty() {
                    let label = Label::new(Some("No events"));
                    label.set_halign(Align::Center);
                    self.agenda.append(&label);
                } else {
                    events.iter().for_each(|e| {
                        let item_box = Box::new(Orientation::Horizontal, 8);
                        item_box.add_css_class("agenda-event-item");

                        let summary = Label::new(Some(&e.summary));
                        summary.set_halign(Align::Start);
                        summary.set_hexpand(true);

                        let duration = Label::new(Some(&e.duration));
                        duration.set_halign(Align::End);

                        item_box.append(&summary);
                        item_box.append(&duration);

                        self.agenda.append(&item_box);
                    });
                }
            }
            Tab::Tasks { cal, past: _ } => {
                let tasks = state.get_tasks(cal.as_deref());
                if tasks.is_empty() {
                    let label = Label::new(Some("No tasks"));
                    label.set_halign(Align::Center);
                    self.agenda.append(&label);
                } else {
                    tasks.iter().for_each(|t| {
                        let item_box = Box::new(Orientation::Horizontal, 8);
                        item_box.add_css_class("agenda-task-item");

                        let summary = Label::new(Some(&t.summary));
                        summary.set_halign(Align::Start);
                        summary.set_hexpand(true);

                        let due = Label::new(Some(&t.duetxt));
                        due.set_halign(Align::End);

                        item_box.append(&summary);
                        item_box.append(&due);

                        self.agenda.append(&item_box);
                    });
                }
            }
        };
        drop(ui);
        drop(state);
    }

    pub fn cycle_calendar(&self, right: bool) {
        let state = self.state.lock().unwrap();
        let cal_names = state.calendar_names();
        let mut ui = self.ui_state.lock().unwrap();
        let new_cal = match ui.selected_cal() {
            Some(cal) => match cal_names.iter().position(|c| *c == cal) {
                Some(idx) => {
                    if right {
                        if idx + 1 >= cal_names.len() {
                            None
                        } else {
                            Some(cal_names[idx + 1].clone())
                        }
                    } else if idx == 0 {
                        None
                    } else {
                        Some(cal_names[idx - 1].clone())
                    }
                }
                None => None,
            },
            None => {
                if right {
                    Some(cal_names.first().expect("No calendars found. Which is really weird because this program should not start if there are no calendars.").clone())
                } else {
                    Some(cal_names.last().expect("No calendars found. Which is really weird because this program should not start if there are no calendars.").clone())
                }
            }
        };
        ui.set_selected_cal(new_cal);
        drop(ui);
        drop(state);
        self.update();
    }

    pub fn toggle_tab(&self) {
        let mut ui = self.ui_state.lock().unwrap();
        let cal = ui.selected_cal();
        ui.tab = match &ui.tab {
            Tab::Events { .. } => Tab::Tasks { past: false, cal },
            Tab::Tasks { .. } => Tab::Events {
                show_tasks: false,
                cal,
            },
        };
        drop(ui);
        self.update();
    }
}
