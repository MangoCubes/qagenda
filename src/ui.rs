use std::sync::{Arc, Mutex};

use chrono::Local;
use gtk4::prelude::BoxExt;
use gtk4::{Align, Box, Separator};
use gtk4::{
    Application, ApplicationWindow, CssProvider, EventControllerKey, Label, Orientation,
    gdk::{Display, Key},
    gio::prelude::ApplicationExt,
    glib::Propagation,
    prelude::*,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use serde::{Deserialize, Serialize};

use crate::{config::Config, state::State};

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
        .title("QCal")
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
        update_view(&title, ui.selected_cal().as_deref());
    }

    let ckey = EventControllerKey::new();
    let app2 = app.clone();
    let window2 = window.clone();
    let state2 = state.clone();
    let ui_state2 = ui_state.clone();
    let title2 = title.clone();

    ckey.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == Key::Escape {
            window2.set_visible(false);
            window2.set_sensitive(false);
            app2.quit();
            Propagation::Stop
        } else if keyval == Key::Right || keyval == Key::Left {
            let state = state2.lock().unwrap();
            let cal_names = state.calendar_names();
            let mut ui = ui_state2.lock().unwrap();
            let new_cal = {
                match ui.selected_cal() {
                    Some(cal) => match cal_names.iter().position(|c| *c == cal) {
                        Some(idx) => {
                            if keyval == Key::Right {
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
                        if keyval == Key::Right {
                            Some(cal_names.first().expect("No calendars found. Which is really weird because this program should not start if there are no calendars.").clone())
                        } else {
                            Some(cal_names.last().expect("No calendars found. Which is really weird because this program should not start if there are no calendars.").clone())
                        }
                    }
                }
            };
            update_view(&title2, new_cal.as_deref());
            ui.set_selected_cal(new_cal);
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });

    window.add_controller(ckey);

    window.set_child(Some(&vbox));
    window.present();
}

fn update_view(title: &Label, cal: Option<&str>) {
    let title_text = match cal {
        Some(name) => format!("Agenda - {}", name),
        None => "Agenda (All calendars)".to_string(),
    };
    title.set_text(&title_text);
}
