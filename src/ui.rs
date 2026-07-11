use std::sync::{Arc, Mutex};

use chrono::Local;
use gtk4::{
    Application, ApplicationWindow, Box, CssProvider, EventControllerKey, Label, Orientation,
    Separator,
    gdk::Display,
    gio::prelude::ApplicationExt,
    glib::Propagation,
    prelude::{BoxExt, GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::{
    config::{Config, keybinds::Action},
    state::State,
    ui::{calendar::MonthCalendar, widget::Widget},
};

mod calendar;
pub mod state;
mod widget;

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

    let cal = MonthCalendar::build();
    vbox.append(&cal);

    let divider = Separator::builder().build();
    vbox.append(&divider);

    let widget = Widget::new(ui_state, state);
    vbox.append(&widget.title);
    vbox.append(&widget.cal_indicator);
    vbox.append(&widget.agenda);

    let ckey = EventControllerKey::new();
    let app2 = app.clone();
    let window2 = window.clone();
    let keybinds = config.keybinds.clone();
    let widget2 = widget.clone();

    ckey.connect_key_pressed(move |_, keyval, _, state| {
        if let Some(action) = keybinds.get(&keyval, state) {
            match action {
                Action::Right => {
                    widget2.cycle_calendar(true);
                    Propagation::Stop
                }
                Action::Left => {
                    widget2.cycle_calendar(false);
                    Propagation::Stop
                }
                Action::SectionLeft | Action::SectionRight => {
                    widget2.toggle_tab();
                    Propagation::Stop
                }
                Action::Exit => {
                    window2.set_visible(false);
                    window2.set_sensitive(false);
                    app2.quit();
                    Propagation::Stop
                }
                _ => Propagation::Proceed,
            }
        } else {
            Propagation::Proceed
        }
    });

    window.add_controller(ckey);

    window.set_child(Some(&vbox));
    window.present();
}
