use std::iter;

use chrono::{Datelike, Timelike};
use gtk4::{
    Align, Box, Entry, Grid, Label, Orientation,
    prelude::{BoxExt, EditableExt, EntryExt, GridExt, WidgetExt},
};

use crate::{
    config::keybinds::{Action, Direction},
    state::{State, details::Details, diff::SingleDiff},
    ui::{
        calendar::MonthCalendar,
        state::{
            Focus, Mode, Tab, UIState,
            editor::{
                DateFieldType, DateTimeFieldType, EditItem, EditorField, EditorState, TimeField,
            },
        },
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
        match self.ui_state.mode() {
            Mode::ConfirmExit => {
                self.show_confirm_exit();
                return;
            }
            Mode::Edit(_) => {
                self.show_editor();
                return;
            }
            Mode::Browse => {}
        }
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
            Tab::Events { cal, .. } => self.state.get_events_count(cal.as_deref()),
            Tab::Tasks { cal, past: _ } => self.state.get_tasks_count(cal.as_deref()),
        };

        if item_count > 0 && self.ui_state.current_item() >= item_count {
            self.ui_state.set_current_item(item_count - 1);
        }

        let expanded = self.ui_state.expanded();
        let current = self.ui_state.current_item();

        fn item_box(
            class: &str,
            selected: bool,
            date: &String,
            has_details: bool,
            summary: &String,
        ) -> Box {
            let item_box = Box::new(Orientation::Vertical, 0);
            item_box.add_css_class(class);
            item_box.add_css_class("agenda-item");
            if selected {
                item_box.add_css_class("agenda-item-selected");
            }

            let due = Label::new(Some(&date));
            due.set_halign(Align::End);

            let row = Box::new(Orientation::Horizontal, 8);

            let expandable = Label::new(Some(if has_details { "+" } else { " " }));
            expandable.set_halign(Align::Center);
            expandable.add_css_class("details-indicator");

            row.append(&expandable);

            let summary = Label::new(Some(summary));
            summary.set_halign(Align::Start);
            summary.set_hexpand(true);

            row.append(&summary);
            row.append(&due);

            item_box.append(&row);

            item_box
        }

        fn details(details: &Details) -> Box {
            let panel = Box::new(Orientation::Vertical, 4);

            if let Some(l) = &details.location {
                let label = Label::new(Some(&format!("Where: {}", l)));
                label.set_halign(Align::Start);
                label.add_css_class("detail-row");
                panel.append(&label);
            }

            if let Some(desc) = &details.description {
                let label = Label::new(Some(&format!("Notes: {}", desc)));
                label.set_halign(Align::Start);
                label.set_wrap(true);
                label.add_css_class("detail-row");
                panel.append(&label);
            }

            panel
        }

        match &self.ui_state.tab() {
            Tab::Events { cal, .. } => {
                let events = self.state.get_events(cal.as_deref());
                if events.is_empty() {
                    let label = Label::new(Some("No events"));
                    label.set_halign(Align::Center);
                    self.agenda.append(&label);
                } else {
                    events.iter().enumerate().for_each(|(i, e)| {
                        let selected = i == current;
                        let modified = self.state.pending.get_event(e);
                        let e = modified.unwrap_or(e.clone());
                        let item_box = item_box(
                            "agenda-event-item",
                            selected,
                            &e.duration,
                            e.details.has_details(),
                            &e.summary,
                        );

                        if expanded && selected && e.details.has_details() {
                            item_box.append(&details(&e.details));
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
                    tasks.iter().enumerate().for_each(|(i, t)| {
                        let selected = i == current;
                        let modified = self.state.pending.get_task(t);
                        let t = modified.unwrap_or(t.clone());
                        let completed = self.state.pending.is_task_completed(&t);
                        let item_box = item_box(
                            "agenda-task-item",
                            selected,
                            &t.duetxt,
                            t.details.has_details(),
                            &t.summary,
                        );

                        if completed {
                            item_box.add_css_class("task-completed");
                        }

                        if expanded && selected && t.details.has_details() {
                            item_box.append(&details(&t.details));
                        }

                        self.agenda.append(&item_box);
                    });
                }
            }
        };
    }

    fn show_confirm_exit(&self) {
        while let Some(child) = self.agenda.first_child() {
            self.agenda.remove(&child);
        }

        self.agenda_title.set_text("Pending Changes");

        self.state
            .pending
            .get_changes()
            .iter()
            .for_each(|(cal, msgs)| {
                let cal = Label::new(Some(&format!("[{}]", cal)));
                cal.set_halign(Align::Start);
                cal.add_css_class("section-title");
                self.agenda.append(&cal);

                let grid = Grid::new();

                msgs.iter()
                    .flat_map(|msg| match msg {
                        SingleDiff::Create { summary } | SingleDiff::Delete { summary } => {
                            vec![("-", summary.as_str())]
                        }
                        SingleDiff::Update { summary, changes } => {
                            iter::once(("-", summary.as_str()))
                                .chain(changes.iter().map(|c| ("", c.as_str())))
                                .collect()
                        }
                    })
                    .enumerate()
                    .for_each(|(row, (bullet, text))| {
                        let bullet = Label::new(Some(bullet));
                        bullet.set_halign(Align::Center);
                        bullet.add_css_class("item-bullet");
                        let changes = Label::new(Some(text));
                        changes.set_halign(Align::Start);
                        changes.add_css_class("item-changes");

                        grid.attach(&bullet, 0, row as i32, 1, 1);
                        grid.attach(&changes, 1, row as i32, 1, 1);
                    });

                self.agenda.append(&grid);
            });
        let query = Label::new(Some("Write changes? (y/n/esc)"));
        query.set_halign(Align::Start);
        self.agenda.append(&query);
    }

    pub fn start_creating(&self) {
        let Some(cal) = self.ui_state.selected_cal() else {
            return;
        };
        let editor = match self.ui_state.tab() {
            Tab::Events { .. } => EditorState::new_event(cal),
            Tab::Tasks { .. } => EditorState::new_task(cal),
        };
        self.ui_state.start_new(editor);
    }

    pub fn start_editing_selected(&self) {
        let current = self.ui_state.current_item();
        match self.ui_state.tab() {
            Tab::Events { cal, .. } => {
                let events = self.state.get_events(cal.as_deref());
                if let Some(event) = events.get(current) {
                    let item = self
                        .state
                        .pending
                        .get_event(event)
                        .unwrap_or_else(|| (*event).clone());
                    self.ui_state.start_edit(EditItem::Event(item));
                }
            }
            Tab::Tasks { cal, .. } => {
                let tasks = self.state.get_tasks(cal.as_deref());
                if let Some(task) = tasks.get(current) {
                    let item = self
                        .state
                        .pending
                        .get_task(task)
                        .unwrap_or_else(|| (*task).clone());
                    self.ui_state.start_edit(EditItem::Task(item));
                }
            }
        }
    }

    pub fn save_item(&self, editor: &EditorState) {
        let loc = if editor.location.trim().is_empty() {
            None
        } else {
            Some(editor.location.clone())
        };
        let desc = if editor.desc.trim().is_empty() {
            None
        } else {
            Some(editor.desc.clone())
        };
        let end = editor.end.to_dpt();
        if editor.is_new {
            match &editor.orig {
                EditItem::Event(e) => {
                    self.state.pending.add_event(
                        e.cal.clone(),
                        editor.summary.clone(),
                        editor.start.to_dpt(),
                        end,
                        loc,
                        desc,
                    );
                }
                EditItem::Task(t) => {
                    self.state.pending.add_task(
                        t.cal.clone(),
                        editor.summary.clone(),
                        end,
                        loc,
                        desc,
                    );
                }
            }
        } else {
            match &editor.orig {
                EditItem::Event(orig) => {
                    self.state.pending.update_event(
                        orig,
                        editor.summary.clone(),
                        editor.start.to_dpt(),
                        end,
                        loc,
                        desc,
                    );
                }
                EditItem::Task(orig) => {
                    self.state
                        .pending
                        .update_task(orig, editor.summary.clone(), end, loc, desc);
                }
            }
        }
    }

    fn show_editor(&self) {
        while let Some(child) = self.agenda.first_child() {
            self.agenda.remove(&child);
        }

        let Some(editor) = self.ui_state.editor_state() else {
            return;
        };

        let is_event = matches!(editor.orig, EditItem::Event(_));
        let title = match (editor.is_new, is_event) {
            (true, true) => "New Event",
            (true, false) => "New Task",
            (false, true) => "Edit Event",
            (false, false) => "Edit Task",
        };
        self.agenda_title.set_text(title);

        let active = editor.is_editing();
        let selected_field = editor.selected_field.0;

        fn gen_label(text: &str, selected: bool) -> Box {
            let row = Box::new(Orientation::Horizontal, 8);
            row.add_css_class("editor-field-row");

            if selected {
                row.add_css_class("editor-field-row-selected");
            }
            let label = Label::new(Some(text));
            label.set_halign(Align::Start);
            label.add_css_class("editor-field-label");
            row.append(&label);
            row
        }

        let display_string = |label: EditorField, val: &str| {
            let selected = label == selected_field;
            let row = gen_label(label.label(is_event), selected);
            let entry = if selected && active {
                let entry = Entry::builder().text(val).hexpand(true).build();
                entry.add_css_class("editor-entry");

                let entry2 = entry.clone();
                let ui_state = self.ui_state.clone();
                let widget = self.clone();

                entry.connect_activate(move |_| {
                    let new = entry2.text().to_string();
                    ui_state.editor_stop_write(new, true);
                    widget.update();
                });

                row.append(&entry);
                Some(entry)
            } else {
                let label = Label::new(Some(if val.is_empty() { "(empty)" } else { val }));
                label.set_halign(Align::Start);
                label.set_hexpand(true);
                row.append(&label);
                None
            };

            self.agenda.append(&row);

            if let Some(entry) = entry {
                entry.grab_focus();
                entry.set_position(-1);
            }
        };
        let display_time = |label: EditorField, val: &TimeField| {
            let field = |value: u32, width: usize| {
                let l = Label::new(Some(&format!("{:0width$}", value)));
                l.add_css_class("editor-field");
                l
            };
            let selected = label == selected_field;
            let row = gen_label(label.label(is_event), selected);
            let focused_field = |value: u32, width: i32| -> Option<Entry> {
                if selected && active {
                    let entry = Entry::new();
                    entry.set_text(&value.to_string());
                    entry.set_width_chars(width);
                    entry.set_max_width_chars(width);
                    entry.set_max_length(width);
                    row.append(&entry);
                    Some(entry)
                } else {
                    let label = field(value, width as usize);
                    if selected {
                        label.add_css_class("editor-field-selected");
                    }
                    row.append(&label);
                    None
                }
            };
            let date_divider = || Label::new(Some("/"));
            let time_divider = || Label::new(Some(":"));
            let space_divider = || Label::new(Some(" "));
            self.agenda.append(&row);
            if let Some(entry) = match val {
                TimeField::None => {
                    let label = Label::new(Some("(none)"));
                    row.append(&label);
                    None
                }
                TimeField::Date(naive_date, date_field_type) => match &date_field_type {
                    DateFieldType::Year => {
                        let entry = focused_field(naive_date.year() as u32, 4);
                        row.append(&date_divider());
                        row.append(&field(naive_date.month(), 2));
                        row.append(&date_divider());
                        row.append(&field(naive_date.day(), 2));
                        entry
                    }
                    DateFieldType::Month => {
                        row.append(&field(naive_date.year() as u32, 4));
                        row.append(&date_divider());
                        let entry = focused_field(naive_date.month(), 2);
                        row.append(&date_divider());
                        row.append(&field(naive_date.day(), 2));
                        entry
                    }
                    DateFieldType::Day => {
                        row.append(&field(naive_date.year() as u32, 4));
                        row.append(&date_divider());
                        row.append(&field(naive_date.month(), 2));
                        row.append(&date_divider());
                        let entry = focused_field(naive_date.day(), 2);
                        entry
                    }
                },
                TimeField::DateTime(naive_date_time, date_time_field_type) => {
                    match date_time_field_type {
                        DateTimeFieldType::Year => {
                            let entry = focused_field(naive_date_time.year() as u32, 4);
                            row.append(&date_divider());
                            row.append(&field(naive_date_time.month(), 2));
                            row.append(&date_divider());
                            row.append(&field(naive_date_time.day(), 2));
                            row.append(&space_divider());
                            row.append(&field(naive_date_time.hour(), 2));
                            row.append(&time_divider());
                            row.append(&field(naive_date_time.minute(), 2));
                            entry
                        }
                        DateTimeFieldType::Month => {
                            row.append(&field(naive_date_time.year() as u32, 4));
                            row.append(&date_divider());
                            let entry = focused_field(naive_date_time.month(), 2);
                            row.append(&date_divider());
                            row.append(&field(naive_date_time.day(), 2));
                            row.append(&space_divider());
                            row.append(&field(naive_date_time.hour(), 2));
                            row.append(&time_divider());
                            row.append(&field(naive_date_time.minute(), 2));
                            entry
                        }
                        DateTimeFieldType::Day => {
                            row.append(&field(naive_date_time.year() as u32, 4));
                            row.append(&date_divider());
                            row.append(&field(naive_date_time.month(), 2));
                            row.append(&date_divider());
                            let entry = focused_field(naive_date_time.day(), 2);
                            row.append(&space_divider());
                            row.append(&field(naive_date_time.hour(), 2));
                            row.append(&time_divider());
                            row.append(&field(naive_date_time.minute(), 2));
                            entry
                        }
                        DateTimeFieldType::Hour => {
                            row.append(&field(naive_date_time.year() as u32, 4));
                            row.append(&date_divider());
                            row.append(&field(naive_date_time.month(), 2));
                            row.append(&date_divider());
                            row.append(&field(naive_date_time.day(), 2));
                            row.append(&space_divider());
                            let entry = focused_field(naive_date_time.hour(), 2);
                            row.append(&time_divider());
                            row.append(&field(naive_date_time.minute(), 2));
                            entry
                        }
                        DateTimeFieldType::Minute => {
                            row.append(&field(naive_date_time.year() as u32, 4));
                            row.append(&date_divider());
                            row.append(&field(naive_date_time.month(), 2));
                            row.append(&date_divider());
                            row.append(&field(naive_date_time.day(), 2));
                            row.append(&space_divider());
                            row.append(&field(naive_date_time.hour(), 2));
                            row.append(&time_divider());
                            let entry = focused_field(naive_date_time.minute(), 2);
                            entry
                        }
                    }
                }
            } {
                let ui_state = self.ui_state.clone();
                let widget = self.clone();
                entry.add_css_class("editor-entry");
                entry.grab_focus();
                entry.set_position(-1);
                let entry2 = entry.clone();

                entry.connect_activate(move |_| {
                    let new = entry2.text().to_string();
                    ui_state.editor_stop_write(new, true);
                    widget.update();
                });
            }
        };

        display_string(EditorField::Summary, &editor.summary);
        display_time(EditorField::Start, &editor.start);
        display_time(EditorField::End, &editor.end);
        display_string(EditorField::Location, &editor.location);
        display_string(EditorField::Description, &editor.desc);
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

    pub fn toggle_task(&self) {
        if let Tab::Tasks { past: _, cal } = self.ui_state.tab() {
            let tasks = self.state.get_tasks(cal.as_deref());
            if let Some(task) = tasks.get(self.ui_state.current_item()) {
                self.state.pending.toggle_task(task);
            }
        }
    }

    pub fn handle_action(&self, action: Action) {
        match action {
            Action::SectionUp => {
                self.ui_state.set_focus(Focus::Calendar);
            }
            Action::SectionDown => {
                self.ui_state.set_focus(Focus::Agenda);
            }
            Action::Move(Direction::Left) => {
                if self.ui_state.focus() == Focus::Calendar {
                    self.ui_state.cycle_month(false);
                } else {
                    self.cycle_calendar(false);
                }
            }
            Action::Move(Direction::Right) => {
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
            Action::Move(Direction::Up) => {
                if self.ui_state.focus() == Focus::Agenda {
                    let item_count = match &self.ui_state.tab() {
                        Tab::Events { cal, .. } => self.state.get_events_count(cal.as_deref()),
                        Tab::Tasks { cal, past: _ } => self.state.get_tasks_count(cal.as_deref()),
                    };
                    self.ui_state.cycle_item(false, item_count);
                }
            }
            Action::Move(Direction::Down) => {
                if self.ui_state.focus() == Focus::Agenda {
                    let item_count = match &self.ui_state.tab() {
                        Tab::Events { cal, .. } => self.state.get_events_count(cal.as_deref()),
                        Tab::Tasks { cal, past: _ } => self.state.get_tasks_count(cal.as_deref()),
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
            Action::ToggleComplete => {
                if self.ui_state.focus() == Focus::Agenda {
                    self.toggle_task();
                }
            }
            Action::Edit => {
                if self.ui_state.focus() == Focus::Agenda {
                    self.start_editing_selected();
                }
            }
            Action::Add => {
                if self.ui_state.focus() == Focus::Agenda {
                    self.start_creating();
                }
            }
            _ => {}
        };

        self.update();
    }
}
