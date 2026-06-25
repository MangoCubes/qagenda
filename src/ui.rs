use std::sync::{Arc, Mutex};

use chrono::Local;
use gtk4::prelude::BoxExt;
use gtk4::{
    Application, ApplicationWindow, CssProvider, EventControllerKey, Label, Orientation,
    gdk::{Display, Key},
    gio::prelude::ApplicationExt,
    glib::Propagation,
    prelude::{GtkWindowExt, WidgetExt},
};
use gtk4::{Box, Separator};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::{config::Config, state::State};

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

    let now = Local::now().format("%Y/%m/%d").to_string();
    let date = Label::new(Some(&now));

    let vbox = Box::new(Orientation::Vertical, 0);

    vbox.append(&date);

    let divider = Separator::builder().build();
    vbox.append(&divider);

    window.set_child(Some(&vbox));
    window.present();
}
