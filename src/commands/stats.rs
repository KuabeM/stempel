//! Handler for the statistics subcommand.
//!
//! The main entry point is `stats` which then further decides what to do.

use crate::balance::{BrakeState, DurationDef, TimeBalance};

use chrono::{DateTime, Datelike, Duration, Month, Utc};
use colored::*;
use failure::{format_err, Error};
use itertools::Itertools;
use num_traits::FromPrimitive;

use std::path::Path;

/// Prints a summary of the current storage either for one month.
///
/// Handler for the `stats` sub command.
pub fn stats<P: AsRef<Path>>(storage: P, month: Option<crate::month::Month>) -> Result<(), Error> {
    let year = Utc::now().year();
    let balance = TimeBalance::from_file(&storage, false)?;
    if let Some(m) = month {
        let m = Month::from_u8(m as u8).ok_or_else(|| format_err!("Failed to parse month"))?;
        monthly_stats(&balance, year, m);
    } else {
        let m = Month::from_u32(Utc::now().month())
            .ok_or_else(|| format_err!("Failed to parse current month"))?;
        let default_cfg = crate::balance::Config::default();
        let history = balance.config.as_ref().unwrap_or(&default_cfg).month_stats;
        println!("Here are your stats for the last {} months:", history);
        stats_last_month(&balance, year, m, history);
    }

    println!();
    show_state(&balance);

    Ok(())
}

/// Generate month, year combination for past months and print the respective stats for them.
fn stats_last_month(balance: &TimeBalance, year: i32, month: Month, history: u8) {
    let mut months: Vec<Month> = vec![month];
    let mut years: Vec<i32> = vec![year];
    (0..history).fold(month, |a, _| {
        months.push(a.pred());
        if a.pred().number_from_month() > month.number_from_month() {
            years.push(year - 1);
        } else {
            years.push(year);
        }
        a.pred()
    });
    years.reverse();
    months.reverse();
    log::trace!("Years: {:?}, months: {:?}", years, months);

    for (y, m) in years.iter().zip(months) {
        monthly_stats(balance, *y, m);
    }
}

/// Prints the entries in the `storage` for one `month` grouped by weeks.
fn monthly_stats(balance: &TimeBalance, year: i32, month: Month) {
    let month_entries: Vec<(&DateTime<Utc>, &DurationDef)> =
        balance.month_range(year, month).collect();
    log::trace!("Month {:?}", month);

    if !month_entries.is_empty() {
        println!("{}:", month.name().green());
        let mut cur_w = 0;
        for (week, group) in &month_entries.into_iter().group_by(|e| {
            let week_num = e.0.iso_week().week();
            if week_num != cur_w {
                cur_w = week_num;
            }
            cur_w
        }) {
            let dur = group.fold(chrono::Duration::zero(), |dur, (_, d)| {
                dur.checked_add(&d.into()).unwrap()
            });
            println!(
                "    Week {:2}: {:02}:{:02}h",
                week,
                dur.num_hours(),
                dur.num_minutes() % 60
            );
        }
    }
}

/// Print current state of started work, running and finished breaks.
fn show_state(balance: &TimeBalance) {
    let dur = if let Some((dur, start)) = balance.start_state() {
        println!(
            "Started at {}, worked {:02}:{:02}h since then.",
            start.with_timezone(&chrono::Local).format("%H:%M"),
            dur.num_hours(),
            dur.num_minutes() % 60
        );
        dur
    } else {
        Duration::zero()
    };
    let pause = match balance.break_state() {
        BrakeState::Started(d, s) => {
            println!(
                "You're on a break since {}, total break duration today is {:02}:{:02}h.",
                s.with_timezone(&chrono::Local).format("%H:%M"),
                d.num_hours(),
                d.num_minutes() % 60
            );
            d
        }
        BrakeState::Finished(d) => {
            println!(
                "Your breaks lasted {:02}:{:02}h.",
                d.num_hours(),
                d.num_minutes() % 60
            );
            d
        }
        BrakeState::NotActive => Duration::seconds(0),
    };

    if let Some(daily) = balance.config.as_ref().unwrap_or_default().daily_hours {
        let daily = Duration::hours(daily as i64);
        let remaining = daily - dur + pause;
        if remaining < Duration::zero() {
            println!(
                "You're done for today. You have {:02}:{:02}h overhours.",
                (-remaining).num_hours(),
                (-remaining).num_minutes() % 60
            );
        } else {
            println!(
                "You still need to work {:02}:{:02}h.",
                remaining.num_hours(),
                remaining.num_minutes() % 60,
            );
        }
    }
}
