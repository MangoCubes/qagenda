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
    ui::widget::Widget,
};

mod calendar;
pub mod state;
mod widget;

pub fn build_ui(app: &Application, config: Config, state: State) {
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

    let ui_state = config.init_state.clone();

    let now = Local::now().format("%Y/%m/%d").to_string();
    let date = Label::new(Some(&now));
    date.add_css_class("section-title");

    let vbox = Box::new(Orientation::Vertical, 0);

    vbox.append(&date);

    let divider = Separator::builder().build();
    vbox.append(&divider);

    let widget = Widget::new(ui_state, state);
    vbox.append(&widget.cal_box);

    let divider = Separator::builder().build();
    vbox.append(&divider);

    vbox.append(&widget.agenda_box);

    let ckey = EventControllerKey::new();
    let app2 = app.clone();
    let window2 = window.clone();
    let keybinds = config.keybinds.clone();
    let widget2 = widget.clone();

    ckey.connect_key_pressed(move |_, keyval, _, state| {
        if let Some(action) = keybinds.get(&keyval, state) {
            if *action == Action::Exit {
                window2.set_visible(false);
                window2.set_sensitive(false);
                app2.quit();
            } else {
                widget2.handle_action(*action);
            };
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    });

    window.add_controller(ckey);

    window.set_child(Some(&vbox));
    window.present();
}
