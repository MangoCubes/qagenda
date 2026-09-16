use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
use icalendar::{CalendarDateTime, Component, DatePerhapsTime, Event};

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

pub fn get_naive_datetime(cdt: &CalendarDateTime) -> NaiveDateTime {
    match cdt {
        CalendarDateTime::Floating(ndt) => *ndt,
        CalendarDateTime::Utc(utc) => utc.naive_utc(),
        CalendarDateTime::WithTimezone { date_time, .. } => *date_time,
    }
}

pub fn dpt_to_naive_datetime(dpt: &DatePerhapsTime) -> NaiveDateTime {
    match dpt {
        DatePerhapsTime::Date(d) => d.and_hms_opt(0, 0, 0).unwrap(),
        DatePerhapsTime::DateTime(cdt) => get_naive_datetime(cdt),
    }
}

pub fn is_past_event(event: &Event) -> bool {
    let today = Local::now().date_naive();
    match event.get_end() {
        Some(DatePerhapsTime::Date(end)) => {
            match event.get_start() {
                // If start and end are the same, that means the event lasts for that whole day.
                // For such event to be considered to be in the past, its end date needs to be
                // strictly in the past.
                Some(DatePerhapsTime::Date(start)) if start == end => end < today,
                _ => end <= today,
            }
        }
        Some(DatePerhapsTime::DateTime(end_dt)) => get_naive_date(&end_dt) < today,
        None => false,
    }
}
