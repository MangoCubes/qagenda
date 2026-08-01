use std::collections::HashMap;

use chrono::{Days, Local, NaiveTime, TimeDelta, TimeZone};
use icalendar::{Calendar, Component, DatePerhapsTime, EventLike, Tz};

use crate::{
    state::{
        event::EventItem,
        task::TaskItem,
        utils::{get_naive_date, get_naive_datetime, is_past_event},
    },
    types::UUID,
};

#[derive(Debug, Clone)]
pub struct MiniCal {
    /// Also contains first occurrence of recurring events so that [`MiniCal::past_events`] +
    /// [`MiniCal::events`] = all events without duplicates
    pub events: Vec<EventItem>,
    pub recurring_events: Vec<EventItem>,
    /// If the last occurrence of a recurring event is past the current date, it goes here as well
    pub past_events: Vec<EventItem>,
    pub tasks: Vec<TaskItem>,
    pub completed_tasks: Vec<TaskItem>,
    /// Tasks whose start date is in the future
    pub upcoming_tasks: Vec<TaskItem>,
}

impl MiniCal {
    pub fn from_calendar(
        cal_name: String,
        cal: &Calendar,
        max_recurrence_count: u32,
        max_recurrence_date: u32,
    ) -> Self {
        let today = Local::now().date_naive();
        let (mut events, mut recurring_events, mut past_events) = (vec![], vec![], vec![]);
        let start_window = Tz::LOCAL
            .from_local_datetime(&today.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap()))
            .single()
            .expect("Apparently the local time falls in a fold or a gap in the local time. At least that's what the documentation says. I have no idea what the hell that means. Sorry.");

        let end_window = if max_recurrence_date > 0 {
            Some(
                Tz::LOCAL
                    .from_local_datetime(
                        &today
                            .checked_add_days(Days::new((max_recurrence_date + 1) as u64))
                            .expect("Max recurrence date is too big!")
                            .and_time(NaiveTime::default()),
                    )
                    .single()
                    .expect("Failed to compute end window for recurrence date limit"),
            )
        } else {
            None
        };

        cal.events().for_each(|event| {
            if event.property_value("RRULE").is_some() {
                match event.get_recurrence() {
                    Ok(rrule) => {
                        let result = {
                            let after = rrule.after(start_window);
                            let bounded = match end_window {
                                Some(end) => after.before(end),
                                None => after,
                            };
                            bounded.all(max_recurrence_count as u16)
                        };
                        if result.dates.is_empty() {
                            // All occurrences are in the past
                            past_events.push(EventItem::from(cal_name.clone(), event));
                        } else {
                            let items = match event.get_end() {
                                Some(end) => {
                                    let Some(start) = event.get_start() else {
                                        if is_past_event(event) {
                                            past_events
                                                .push(EventItem::from(cal_name.clone(), event));
                                        } else {
                                            events.push(EventItem::from(cal_name.clone(), event));
                                        };
                                        return;
                                    };
                                    let duration = {
                                        match (start, end) {
                                            (
                                                DatePerhapsTime::Date(s),
                                                DatePerhapsTime::Date(e),
                                            ) => e - s,
                                            (
                                                DatePerhapsTime::DateTime(s),
                                                DatePerhapsTime::DateTime(e),
                                            ) => get_naive_datetime(&e) - get_naive_datetime(&s),
                                            _ => TimeDelta::zero(),
                                        }
                                    };
                                    result
                                        .dates
                                        .iter()
                                        .map(|start| {
                                            let s = start.naive_local();
                                            EventItem::with_custom_time(
                                                cal_name.clone(),
                                                event,
                                                s.into(),
                                                Some((s + duration).into()),
                                            )
                                        })
                                        .collect::<Vec<EventItem>>()
                                }
                                None => {
                                    // No end date? Huh
                                    result
                                        .dates
                                        .iter()
                                        .map(|start| {
                                            EventItem::with_custom_time(
                                                cal_name.clone(),
                                                event,
                                                start.naive_local().into(),
                                                None,
                                            )
                                        })
                                        .collect::<Vec<EventItem>>()
                                }
                            };
                            recurring_events.extend(items);
                        }
                    }
                    Err(e) => {
                        // Treat it as single-off event
                        eprintln!(
                            "Failed to parse recurrence for event {:?}: {}",
                            event.get_summary(),
                            e
                        );
                        if !is_past_event(event) {
                            events.push(EventItem::from(cal_name.clone(), event));
                        } else {
                            past_events.push(EventItem::from(cal_name.clone(), event));
                        }
                    }
                }
            }
            if is_past_event(event) {
                past_events.push(EventItem::from(cal_name.clone(), event));
            } else {
                events.push(EventItem::from(cal_name.clone(), event));
            }
        });

        let (mut completed_tasks, remaining): (Vec<TaskItem>, Vec<TaskItem>) = cal
            .todos()
            .map(|t| TaskItem::new(cal_name.clone(), t))
            .partition(|t| t.completed);
        let (mut upcoming_tasks, mut tasks): (Vec<TaskItem>, Vec<TaskItem>) =
            remaining.into_iter().partition(|t| {
                t.start.as_ref().is_some_and(|s| {
                    let today = Local::now().date_naive();
                    match s {
                        DatePerhapsTime::Date(d) => *d > today,
                        DatePerhapsTime::DateTime(cdt) => get_naive_date(cdt) > today,
                    }
                })
            });

        completed_tasks.sort();
        upcoming_tasks.sort();
        tasks.sort();
        events.sort();
        recurring_events.sort();
        past_events.sort();

        Self {
            events,
            recurring_events,
            past_events,
            tasks,
            completed_tasks,
            upcoming_tasks,
        }
    }

    pub fn active_events(&self) -> Vec<&EventItem> {
        self.events.iter().chain(&self.recurring_events).collect()
    }

    pub fn incomplete_tasks(&self) -> Vec<&TaskItem> {
        self.tasks.iter().collect()
    }

    pub fn get_events(&self) -> HashMap<UUID, EventItem> {
        self.events
            .clone()
            .into_iter()
            .chain(self.past_events.clone())
            .map(|e| (e.uid.clone(), e))
            .collect()
    }

    pub fn get_tasks(&self) -> HashMap<UUID, TaskItem> {
        self.tasks
            .clone()
            .into_iter()
            .chain(self.completed_tasks.clone())
            .chain(self.upcoming_tasks.clone())
            .map(|e| (e.uid.clone(), e))
            .collect()
    }

    pub fn incomplete_tasks_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn active_events_count(&self) -> usize {
        self.events.len() + self.recurring_events.len()
    }
}
