use chrono::{Datelike, Days, Local, NaiveDate, TimeDelta};
use gtk4::{
    Grid, Label,
    prelude::{Cast, GridExt, WidgetExt},
};

pub struct MonthCalendar;

impl MonthCalendar {
    pub fn build() -> Grid {
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

        let today = Local::now().date_naive();
        let year = today.year();
        let month = today.month();
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

        (0..35).for_each(|i| {
            let date = grid_first.checked_add_days(Days::new(i as u64)).unwrap();
            let col = (i % 7) as i32;
            let row = (i / 7 + 1) as i32;

            let label = Label::new(None);
            label.add_css_class("cal-day");
            grid.attach(&label, col, row, 1, 1);

            label.set_label(&date.day().to_string());

            if date < first || date >= next {
                label.add_css_class("cal-other-month");
            } else {
                label.remove_css_class("cal-other-month");
            }

            if date == today {
                label.add_css_class("cal-today");
            } else {
                label.remove_css_class("cal-today");
            }
        });
        grid
    }

    pub fn update(grid: &Grid, year: i32, month: u32) {
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

        // 5 weeks needed to show a whole month
        // TODO: Handle February where Feb 1st is on the start of week
        (0..35).for_each(|i| {
            let date = grid_first.checked_add_days(Days::new(i as u64)).unwrap();
            let col = (i % 7) as i32;
            let row = (i / 7 + 1) as i32;

            let label = grid
                .child_at(col, row)
                .unwrap()
                .downcast::<Label>()
                .expect("Grid child should be a Label");

            label.set_label(&date.day().to_string());

            if date < first || date >= next {
                label.add_css_class("cal-other-month");
            } else {
                label.remove_css_class("cal-other-month");
            }

            if date == today {
                label.add_css_class("cal-today");
            } else {
                label.remove_css_class("cal-today");
            }
        });
    }
}
