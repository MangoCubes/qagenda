use std::process::Command;

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
    error,
    state::State,
    ui::{state::Mode, widget::Widget},
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
    window.set_keyboard_mode(if config.allow_unfocused {
        KeyboardMode::OnDemand
    } else {
        KeyboardMode::Exclusive
    });

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
    let on_close = config.on_close.clone();
    let on_write = config.on_write.clone();

    fn run_cmd(cmd: &String) {
        match Command::new("sh").arg("-c").arg(cmd).spawn() {
            Ok(child) => println!("Running child process {}.", child.id()),
            Err(err) => error!("Failed to run command: {}", err),
        };
    }
    ckey.connect_key_pressed(move |_, keyval, _, state| match widget2.ui_state.mode() {
        Mode::Edit(editor) => {
            if editor.is_editing() {
                keybinds
                    .get(&keyval, state)
                    .map_or(Propagation::Proceed, |action| match action {
                        Action::Exit => {
                            widget2.ui_state.editor_stop_write(String::new(), false);
                            widget2.update();
                            Propagation::Stop
                        }
                        _ => Propagation::Proceed,
                    })
            } else {
                keybinds
                    .get(&keyval, state)
                    .map_or(Propagation::Proceed, |a| {
                        match a {
                            Action::Move(dir) => {
                                widget2.ui_state.editor_move_field(dir);
                                widget2.update();
                            }
                            Action::Edit => {
                                widget2.ui_state.editor_write();
                                widget2.update();
                            }
                            Action::ToggleComplete => {
                                if let Some(editor) = widget2.ui_state.editor_state() {
                                    widget2.save_item(&editor);
                                }
                                widget2.ui_state.stop_edit();
                                widget2.update();
                            }
                            Action::Exit => {
                                widget2.ui_state.stop_edit();
                                widget2.update();
                            }
                            Action::Expand => {
                                widget2.ui_state.editor_cycle_time();
                                widget2.update();
                            }
                            _ => {}
                        };
                        Propagation::Stop
                    })
            }
        }
        Mode::ConfirmExit => keybinds
            .get(&keyval, state)
            .map_or(Propagation::Proceed, |action| {
                match action {
                    Action::Yes => {
                        // TODO: write changes
                        if let Some(cmd) = &on_write {
                            run_cmd(cmd);
                        }

                        window2.set_visible(false);
                        window2.set_sensitive(false);
                        app2.quit();
                    }
                    Action::No => {
                        if let Some(cmd) = &on_close {
                            run_cmd(cmd);
                        }
                        window2.set_visible(false);
                        window2.set_sensitive(false);
                        app2.quit();
                    }
                    Action::Exit => {
                        widget2.ui_state.set_confirming_exit(false);
                        widget2.update();
                    }
                    _ => {}
                }
                Propagation::Stop
            }),
        Mode::Browse => keybinds
            .get(&keyval, state)
            .map_or(Propagation::Proceed, |action| {
                if *action == Action::Exit {
                    if widget2.state.pending.has_changes() {
                        widget2.ui_state.set_confirming_exit(true);
                        widget2.update();
                    } else {
                        if let Some(cmd) = &on_close {
                            run_cmd(cmd);
                        }
                        window2.set_visible(false);
                        window2.set_sensitive(false);
                        app2.quit();
                    }
                } else {
                    widget2.handle_action(*action);
                }
                Propagation::Stop
            }),
    });

    window.add_controller(ckey);

    window.set_child(Some(&vbox));
    window.present();
}
