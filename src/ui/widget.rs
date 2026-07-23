use std::iter;

use gtk4::{
    Align, Box, Grid, Label, Orientation, Separator,
    prelude::{BoxExt, WidgetExt},
};

use crate::{
    config::keybinds::Action,
    state::State,
    ui::{
        calendar::MonthCalendar,
        state::{Focus, Tab, UIState},
    },
};

#[derive(Clone)]
pub struct Widget {
    pub cal_box: Box,
    pub cal_grid: Grid,
    pub cal_title: Label,
    pub agenda_box: Box,
    pub agenda: Box,
    pub agenda_title: Label,
    pub cal_indicator: Box,
    pub ui_state: UIState,
    pub state: State,
}

impl Widget {
    pub fn new(ui_state: UIState, state: State) -> Self {
        let cal_title = Label::new(None);
        cal_title.set_halign(Align::Center);
        cal_title.add_css_class("section-title");

        let cal_grid = MonthCalendar::build();
        let cal_box = Box::new(Orientation::Vertical, 4);
        cal_box.add_css_class("section-box");
        cal_box.append(&cal_title);
        cal_box.append(&cal_grid);

        let title = Label::new(None);
        title.set_halign(Align::Start);
        title.add_css_class("section-title");

        let cal_indicator = Box::new(Orientation::Horizontal, 4);
        cal_indicator.set_halign(Align::Fill);
        cal_indicator.set_hexpand(true);
        cal_indicator.set_homogeneous(true);

        let agenda = Box::new(Orientation::Vertical, 4);

        let agenda_box = Box::new(Orientation::Vertical, 4);
        agenda_box.add_css_class("section-box");
        agenda_box.append(&title);
        agenda_box.append(&cal_indicator);
        agenda_box.append(&agenda);

        let widget = Self {
            cal_box,
            cal_grid,
            cal_title,
            agenda_box,
            agenda,
            agenda_title: title,
            cal_indicator,
            ui_state,
            state,
        };

        widget.init_indicators();
        widget.update();
        widget
    }

    fn init_indicators(&self) {
        self.state.calendar_names().iter().for_each(|_| {
            let box_item = Box::new(Orientation::Horizontal, 0);
            box_item.set_size_request(-1, -1);
            box_item.set_halign(Align::Fill);
            box_item.set_hexpand(true);
            box_item.add_css_class("cal-indicator");
            self.cal_indicator.append(&box_item);
        });
    }

    pub fn update(&self) {
        match self.ui_state.focus() {
            Focus::Calendar => {
                self.cal_box.add_css_class("focused-section");
                self.cal_box.remove_css_class("unfocused-section");
                self.agenda_box.add_css_class("unfocused-section");
                self.agenda_box.remove_css_class("focused-section");
            }
            Focus::Agenda => {
                self.agenda_box.add_css_class("focused-section");
                self.agenda_box.remove_css_class("unfocused-section");
                self.cal_box.add_css_class("unfocused-section");
                self.cal_box.remove_css_class("focused-section");
            }
        }

        MonthCalendar::update(&self.cal_grid, self.ui_state.year(), self.ui_state.month());

        self.cal_title.set_text(&format!(
            "{}/{}",
            self.ui_state.year(),
            self.ui_state.month()
        ));

        let tab_name = match self.ui_state.tab() {
            Tab::Events { .. } => "Events",
            Tab::Tasks { .. } => "Tasks",
        };
        let agenda_text = match self.ui_state.selected_cal().as_deref() {
            Some(name) => format!("{} - {}", tab_name, name),
            None => format!("{} (All calendars)", tab_name),
        };
        self.agenda_title.set_text(&agenda_text);

        let selected = self.ui_state.selected_cal();
        let show_all = selected.is_none();

        self.state
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

        let item_count = match &self.ui_state.tab() {
            Tab::Events { cal, .. } => self.state.get_events(cal.as_deref()).len(),
            Tab::Tasks { cal, past: _ } => self.state.get_tasks(cal.as_deref()).len(),
        };

        if item_count > 0 && self.ui_state.current_item() >= item_count {
            self.ui_state.set_current_item(item_count - 1);
        }

        match &self.ui_state.tab() {
            Tab::Events { cal, .. } => {
                let events = self.state.get_events(cal.as_deref());
                if events.is_empty() {
                    let label = Label::new(Some("No events"));
                    label.set_halign(Align::Center);
                    self.agenda.append(&label);
                } else {
                    let expanded = self.ui_state.expanded();
                    events.iter().enumerate().for_each(|(i, e)| {
                        let selected = self.ui_state.focus() == Focus::Agenda
                            && i == self.ui_state.current_item();
                        let expand = expanded && selected;

                        let item_box = Box::new(Orientation::Vertical, 0);
                        item_box.add_css_class("agenda-event-item");
                        item_box.add_css_class("agenda-item");
                        if selected {
                            item_box.add_css_class("agenda-item-selected");
                        }

                        let row = Box::new(Orientation::Horizontal, 8);

                        let summary = Label::new(Some(&e.summary));
                        summary.set_halign(Align::Start);
                        summary.set_hexpand(true);

                        let more = Label::new(Some(if e.details.is_some() { "+" } else { " " }));
                        more.set_halign(Align::Center);
                        more.add_css_class("details-indicator");
                        row.append(&more);
                        row.append(&summary);
                        item_box.append(&row);

                        if expand {
                            let details = Box::new(Orientation::Vertical, 4);

                            if let Some(d) = &e.details {
                                if let Some(l) = &d.location {
                                    let label = Label::new(Some(&format!("Where: {}", l)));
                                    label.set_halign(Align::Start);
                                    label.add_css_class("detail-row");
                                    details.append(&label);
                                }

                                if let Some(desc) = &d.description {
                                    let label = Label::new(Some(&format!("Notes: {}", desc)));
                                    label.set_halign(Align::Start);
                                    label.set_wrap(true);
                                    label.add_css_class("detail-row");
                                    details.append(&label);
                                }
                            }

                            item_box.append(&details);
                        }

                        self.agenda.append(&item_box);
                    });
                }
            }
            Tab::Tasks { cal, past: _ } => {
                let tasks = self.state.get_tasks(cal.as_deref());
                if tasks.is_empty() {
                    let label = Label::new(Some("No tasks"));
                    label.set_halign(Align::Center);
                    self.agenda.append(&label);
                } else {
                    let expanded = self.ui_state.expanded();
                    tasks.iter().enumerate().for_each(|(i, t)| {
                        let selected = self.ui_state.focus() == Focus::Agenda
                            && i == self.ui_state.current_item();
                        let expand = expanded && selected;

                        let item_box = Box::new(Orientation::Vertical, 0);
                        item_box.add_css_class("agenda-task-item");
                        item_box.add_css_class("agenda-item");
                        if selected {
                            item_box.add_css_class("agenda-item-selected");
                        }

                        let row = Box::new(Orientation::Horizontal, 8);

                        let summary = Label::new(Some(&t.summary));
                        summary.set_halign(Align::Start);
                        summary.set_hexpand(true);

                        let due = Label::new(Some(&t.duetxt));
                        due.set_halign(Align::End);

                        let more = Label::new(Some(if t.details.is_some() { "+" } else { " " }));
                        more.set_halign(Align::Center);
                        more.add_css_class("details-indicator");
                        row.append(&more);
                        row.append(&summary);
                        row.append(&due);
                        item_box.append(&row);

                        if expand {
                            let details = Box::new(Orientation::Vertical, 4);

                            if let Some(d) = &t.details {
                                if let Some(l) = &d.location {
                                    let label = Label::new(Some(&format!("Where: {}", l)));
                                    label.set_halign(Align::Start);
                                    label.add_css_class("detail-row");
                                    details.append(&label);
                                }

                                if let Some(desc) = &d.description {
                                    let label = Label::new(Some(&format!("Notes: {}", desc)));
                                    label.set_halign(Align::Start);
                                    label.set_wrap(true);
                                    label.add_css_class("detail-row");
                                    details.append(&label);
                                }
                            }

                            item_box.append(&details);
                        }

                        self.agenda.append(&item_box);
                    });
                }
            }
        };
    }

    pub fn cycle_calendar(&self, right: bool) {
        let cal_names = self.state.calendar_names();
        let new_cal = match self.ui_state.selected_cal() {
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
        self.ui_state.set_selected_cal(new_cal);
    }

    pub fn handle_action(&self, action: Action) {
        match action {
            Action::SectionUp => {
                self.ui_state.set_focus(Focus::Calendar);
            }
            Action::SectionDown => {
                self.ui_state.set_focus(Focus::Agenda);
            }
            Action::Left => {
                if self.ui_state.focus() == Focus::Calendar {
                    self.ui_state.cycle_month(false);
                } else {
                    self.cycle_calendar(false);
                }
            }
            Action::Right => {
                if self.ui_state.focus() == Focus::Calendar {
                    self.ui_state.cycle_month(true);
                } else {
                    self.cycle_calendar(true);
                }
            }
            Action::SectionLeft | Action::SectionRight => {
                if self.ui_state.focus() == Focus::Agenda {
                    self.ui_state.toggle_tab();
                }
            }
            Action::Up => {
                if self.ui_state.focus() == Focus::Agenda {
                    let item_count = match &self.ui_state.tab() {
                        Tab::Events { cal, .. } => self.state.get_events(cal.as_deref()).len(),
                        Tab::Tasks { cal, past: _ } => self.state.get_tasks(cal.as_deref()).len(),
                    };
                    self.ui_state.cycle_item(false, item_count);
                }
            }
            Action::Down => {
                if self.ui_state.focus() == Focus::Agenda {
                    let item_count = match &self.ui_state.tab() {
                        Tab::Events { cal, .. } => self.state.get_events(cal.as_deref()).len(),
                        Tab::Tasks { cal, past: _ } => self.state.get_tasks(cal.as_deref()).len(),
                    };
                    self.ui_state.cycle_item(true, item_count);
                }
            }
            Action::Reset => {
                if self.ui_state.focus() == Focus::Calendar {
                    self.ui_state.reset_month();
                } else {
                    self.ui_state.set_selected_cal(None);
                }
            }
            Action::Expand => {
                if self.ui_state.focus() == Focus::Agenda {
                    self.ui_state.toggle_details();
                }
            }
            _ => {}
        };
        self.update();
    }
}
