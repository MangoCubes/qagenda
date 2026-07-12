use chrono::{Datelike, Days, Local, NaiveDate, TimeDelta};
use gtk4::{
    Grid, Label,
    prelude::{GridExt, WidgetExt},
};

pub struct MonthCalendar;

impl MonthCalendar {
    pub fn build() -> Grid {
        let grid = Grid::new();
        grid.set_column_homogeneous(true);
        grid.set_row_spacing(2);
        grid.set_column_spacing(4);

        let today = Local::now().date_naive();
        Self::update(&grid, today.year(), today.month());
        grid
    }

    pub fn update(grid: &Grid, year: i32, month: u32) {
        while let Some(child) = grid.first_child() {
            grid.remove(&child);
        }

        let today = Local::now().date_naive();
        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };

        let grid_first = first
            .checked_sub_signed(TimeDelta::days(
                first.weekday().num_days_from_sunday() as i64
            ))
            .unwrap();

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
            label.add_css_class("cal-day");
            if date < first || date >= next {
                label.add_css_class("cal-other-month");
            }
            if date == today {
                label.add_css_class("cal-today");
            }
            grid.attach(&label, col, row, 1, 1);
        });
    }
}
