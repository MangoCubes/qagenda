use std::sync::{Arc, Mutex};

use chrono::{Datelike, Days, Local, NaiveDate, TimeDelta};
use gtk4::prelude::BoxExt;
use gtk4::{Align, Box, Grid, Separator};
use gtk4::{
    Application, ApplicationWindow, CssProvider, EventControllerKey, Label, Orientation,
    gdk::Display, gio::prelude::ApplicationExt, glib::Propagation, prelude::*,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use serde::{Deserialize, Serialize};

use crate::{
    config::{Config, keybinds::Action},
    state::State,
};

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
    #[default]
    Week,
    Month,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Focus {
    #[default]
    Agenda,
    Calendar,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct UIState {
    pub tab: Tab,
    pub cal: DisplayType,
    pub focus: Focus,
}

impl UIState {
    pub fn selected_cal(&self) -> Option<String> {
        match &self.tab {
            Tab::Tasks { cal, .. } => cal.clone(),
            Tab::Events { cal, .. } => cal.clone(),
        }
    }

    pub fn set_selected_cal(&mut self, cal: Option<String>) {
        match &mut self.tab {
            Tab::Tasks { cal: c, .. } => *c = cal,
            Tab::Events { cal: c, .. } => *c = cal,
        }
    }
}

pub fn build_ui(app: &Application, config: Config, s: State) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title(env!("CARGO_PKG_NAME"))
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_keyboard_mode(KeyboardMode::OnDemand);

    let (top, bottom, left, right) = config.get_edges();
    window.set_anchor(Edge::Top, top);
    window.set_anchor(Edge::Bottom, bottom);
    window.set_anchor(Edge::Left, left);
    window.set_anchor(Edge::Right, right);

    let base_provider = CssProvider::new();
    base_provider.load_from_data(&config.css);
    let display = Display::default().unwrap();
    gtk4::style_context_add_provider_for_display(
        &display,
        &base_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let state = Arc::new(Mutex::new(s.clone()));
    let ui_state = Arc::new(Mutex::new(config.init_state.clone()));

    let now = Local::now().format("%Y/%m/%d").to_string();
    let date = Label::new(Some(&now));

    let vbox = Box::new(Orientation::Vertical, 0);

    vbox.append(&date);

    let divider = Separator::builder().build();
    vbox.append(&divider);

    let cal = build_month_calendar();
    vbox.append(&cal);

    let divider = Separator::builder().build();
    vbox.append(&divider);

    let title = Label::new(None);
    title.set_halign(Align::Start);
    title.add_css_class("section-title");
    vbox.append(&title);

    let agenda = Box::new(Orientation::Vertical, 4);
    vbox.append(&agenda);

    {
        let ui = ui_state.lock().unwrap();
        let st = state.lock().unwrap();
        update_view(&agenda, &title, &ui, &st);
    }

    let ckey = EventControllerKey::new();
    let app2 = app.clone();
    let window2 = window.clone();
    let state2 = state.clone();
    let ui_state2 = ui_state.clone();
    let title2 = title.clone();
    let agenda2 = agenda.clone();
    let keybinds = config.keybinds.clone();

    ckey.connect_key_pressed(move |_, keyval, _, state| {
        if let Some(action) = keybinds.get(&keyval, state) {
            match action {
                Action::Right | Action::Left => {
                    let state = state2.lock().unwrap();
                    let cal_names = state.calendar_names();
                    let mut ui = ui_state2.lock().unwrap();
                    let new_cal = {
                        match ui.selected_cal() {
                            Some(cal) => match cal_names.iter().position(|c| *c == cal) {
                                Some(idx) => {
                                    if *action == Action::Right {
                                        if idx + 1 >= cal_names.len() {
                                            None
                                        } else {
                                            Some(cal_names[idx + 1].clone())
                                        }
                                    } else {
                                        if idx <= 0 {
                                            None
                                        } else {
                                            Some(cal_names[idx - 1].clone())
                                        }
                                    }
                                }
                                None => None,
                            },
                            None => {
                                if *action == Action::Right {
                                    Some(cal_names.first().expect("No calendars found. Which is really weird because this program should not start if there are no calendars.").clone())
                                } else {
                                    Some(cal_names.last().expect("No calendars found. Which is really weird because this program should not start if there are no calendars.").clone())
                                }
                            }
                        }
                    };
                    ui.set_selected_cal(new_cal);
                    update_view(&agenda2, &title2, &ui, &state);
                    Propagation::Stop
                }
                Action::SectionLeft | Action::SectionRight => {
                    let state = state2.lock().unwrap();
                    let mut ui = ui_state2.lock().unwrap();
                    let cal = ui.selected_cal();
                    match &ui.tab {
                        Tab::Events { .. } => {
                            ui.tab = Tab::Tasks { past: true, cal };
                        }
                        Tab::Tasks { .. } => {
                            ui.tab = Tab::Events { show_tasks: false, cal };
                        }
                    }
                    update_view(&agenda2, &title2, &ui, &state);
                    Propagation::Stop
                }
                Action::Exit => {
                    window2.set_visible(false);
                    window2.set_sensitive(false);
                    app2.quit();
                    Propagation::Stop
                },
                _ => {
                    Propagation::Proceed
                },
            }
        } else {
            Propagation::Proceed
        }
    });

    window.add_controller(ckey);

    window.set_child(Some(&vbox));
    window.present();
}

fn build_month_calendar() -> Grid {
    let today = Local::now().date_naive();
    let first = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let next = NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1).unwrap();

    let grid_first = first
        .checked_sub_signed(TimeDelta::days(
            first.weekday().num_days_from_sunday() as i64
        ))
        .unwrap();

    let grid = Grid::new();
    grid.set_column_homogeneous(true);
    grid.set_row_spacing(2);
    grid.set_column_spacing(4);

    ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]
        .iter()
        .enumerate()
        .for_each(|(col, name)| {
            let label = Label::new(Some(name));
            label.add_css_class("cal-header");
            grid.attach(&label, col as i32, 0, 1, 1);
        });

    // 5 weeks needed to show a whole month
    // TODO: Handle February where Feb 1st is on the start of week
    (0..35).for_each(|i| {
        let date = grid_first.checked_add_days(Days::new(i as u64)).unwrap();
        let col = (i % 7) as i32;
        let row = (i / 7 + 1) as i32; // +1 for header row

        let label = Label::new(Some(&date.day().to_string()));
        if date < first || date >= next {
            label.add_css_class("cal-other-month");
        }
        if date == today {
            label.add_css_class("cal-today");
        }
        grid.attach(&label, col, row, 1, 1);
    });

    grid
}

fn update_view(agenda: &Box, title: &Label, ui: &UIState, state: &State) {
    let tab_name = match ui.tab {
        Tab::Events { .. } => "Events",
        Tab::Tasks { .. } => "Tasks",
    };
    let title_text = match ui.selected_cal().as_deref() {
        Some(name) => format!("Agenda ({}) - {}", tab_name, name),
        None => format!("Agenda ({}) (All calendars)", tab_name),
    };
    title.set_text(&title_text);

    while let Some(child) = agenda.first_child() {
        agenda.remove(&child);
    }

    match &ui.tab {
        Tab::Events { cal, .. } => {
            let events = state.get_events(cal.as_deref());
            if events.is_empty() {
                let label = Label::new(Some("No events"));
                label.set_halign(Align::Start);
                agenda.append(&label);
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

                    agenda.append(&item_box);
                })
            }
        }
        Tab::Tasks { cal, past: _ } => {
            let tasks = state.get_tasks(cal.as_deref());
            if tasks.is_empty() {
                let label = Label::new(Some("No tasks"));
                label.set_halign(Align::Start);
                agenda.append(&label);
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

                    agenda.append(&item_box);
                });
            }
        }
    }
}
