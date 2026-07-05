use chrono::{Datelike, Local, NaiveDate};
use icalendar::{CalendarDateTime, DatePerhapsTime};

pub fn format_date_perhaps_time(dpt: &DatePerhapsTime) -> String {
    let year = match dpt {
        DatePerhapsTime::Date(d) => d.year(),
        DatePerhapsTime::DateTime(cdt) => get_naive_date(cdt).year(),
    };
    if year == Local::now().year() {
        match dpt {
            DatePerhapsTime::Date(d) => d.format("%m/%d").to_string(),
            DatePerhapsTime::DateTime(cdt) => match cdt {
                CalendarDateTime::Floating(ndt) => ndt.format("%m/%d %H:%M").to_string(),
                CalendarDateTime::Utc(utc) => utc.format("%m/%d %H:%M UTC").to_string(),
                CalendarDateTime::WithTimezone { date_time, .. } => {
                    date_time.format("%m/%d %H:%M").to_string()
                }
            },
        }
    } else {
        match dpt {
            DatePerhapsTime::Date(d) => d.format("%Y/%m/%d").to_string(),
            DatePerhapsTime::DateTime(cdt) => match cdt {
                CalendarDateTime::Floating(ndt) => ndt.format("%Y/%m/%d %H:%M").to_string(),
                CalendarDateTime::Utc(utc) => utc.format("%Y/%m/%d %H:%M UTC").to_string(),
                CalendarDateTime::WithTimezone { date_time, .. } => {
                    date_time.format("%Y/%m/%d %H:%M").to_string()
                }
            },
        }
    }
}

pub fn format_time_only(cdt: &CalendarDateTime) -> String {
    match cdt {
        CalendarDateTime::Floating(ndt) => ndt.format("%H:%M").to_string(),
        CalendarDateTime::Utc(utc) => utc.format("%H:%M UTC").to_string(),
        CalendarDateTime::WithTimezone { date_time, .. } => date_time.format("%H:%M").to_string(),
    }
}

pub fn get_naive_date(cdt: &CalendarDateTime) -> NaiveDate {
    match cdt {
        CalendarDateTime::Floating(ndt) => ndt.date(),
        CalendarDateTime::Utc(utc) => utc.date_naive(),
        CalendarDateTime::WithTimezone { date_time, .. } => date_time.date(),
    }
}
