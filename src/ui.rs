use std::sync::{Arc, Mutex};

use chrono::Local;
use gtk4::prelude::BoxExt;
use gtk4::{
    Application, ApplicationWindow, CssProvider, EventControllerKey, Label, Orientation,
    gdk::{Display, Key},
    gio::prelude::ApplicationExt,
    glib::Propagation,
    prelude::*,
};
use gtk4::{Box, PolicyType, ScrolledWindow, Separator};
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

pub fn build_ui(app: &Application, config: Config, state: State) {
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

    let key_controller = EventControllerKey::new();
    let a = app.clone();
    let w = window.clone();

    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == Key::Escape {
            w.set_visible(false);
            w.set_sensitive(false);
            a.quit();
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });

    window.add_controller(key_controller);

    let items = Arc::new(Mutex::new(state.clone()));
    let ui_state = Arc::new(Mutex::new(config.init_state.clone()));

    let now = Local::now().format("%Y/%m/%d").to_string();
    let date = Label::new(Some(&now));

    let vbox = Box::new(Orientation::Vertical, 0);

    vbox.append(&date);

    let divider = Separator::builder().build();
    vbox.append(&divider);

    let divider = Separator::builder().build();
    vbox.append(&divider);

    let title = Label::new(Some("Agenda"));
    title.set_halign(gtk4::Align::Start);
    title.add_css_class("section-title");
    vbox.append(&title);

    let agenda = Box::new(Orientation::Vertical, 4);
    for summary in state.get_agenda() {
        let label = Label::new(Some(&summary));
        label.set_halign(gtk4::Align::Start);
        label.add_css_class("agenda-item");
        agenda.append(&label);
    }

    window.set_child(Some(&vbox));
    window.present();
}
