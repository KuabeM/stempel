//! Handler for the statistics subcommand.
//!
//! The main entry point is `stats` which then further decides what to do.

use crate::balance::{Config, DurationDef, TimeBalance};

use crate::errors::*;
use crate::month;
use chrono::{DateTime, Datelike, Duration, Local, Month, Utc, NaiveDate, NaiveDateTime};
use colored::*;
use itertools::Itertools;
use num_traits::FromPrimitive;

use std::path::Path;

/// Prints a summary of the current storage either for one month.
///
/// Handler for the `stats` sub command.
pub fn stats<P: AsRef<Path>>(storage: P, month: Option<month::Month>) -> Result<()> {
    let year = Utc::now().year();
    let balance = TimeBalance::from_file(&storage, false)?;
    if let Some(m) = month {
        let m = Month::from_u8(m as u8).ok_or_else(|| eyre!("Failed to parse {} into month", m))?;
        monthly_stats(&balance, year, m)?;
    } else {
        let m = Month::from_u32(Utc::now().month())
            .ok_or_else(|| eyre!("Failed to parse current month"))?;
        let default_cfg = Config::default();
        let history = balance.config.as_ref().unwrap_or(&default_cfg).month_stats;
        println!("Here are your stats for the last {} months:", history);
        stats_last_month(&balance, year, m, history)?;
    }

    println!();
    show_state(&balance);
    avg_start_time(&balance)?;

    Ok(())
}

/// Generate month, year combination for past months and print the respective stats for them.
fn stats_last_month(balance: &TimeBalance, year: i32, month: Month, history: u8) -> Result<()> {
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
        monthly_stats(balance, *y, m)?;
    }
    Ok(())
}

/// Prints the entries in the `storage` for one `month` grouped by weeks.
fn monthly_stats(balance: &TimeBalance, year: i32, month: Month) -> Result<()> {
    let month_entries: Vec<(&DateTime<Utc>, &DurationDef)> =
        balance.month_range(year, month)?.collect();
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
    Ok(())
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
    let break_state = balance.break_state();
    let break_str = break_state
        .breaks
        .iter()
        .fold(String::new(), |acc, (s, d)| {
            format!(
                "{}{} for {:02}:{:02}h, ",
                acc,
                s.with_timezone(&Local).time().format("%H:%M"),
                d.num_hours(),
                d.num_minutes() % 60
            )
        });
    let pause = if let Some(start) = break_state.current {
        println!(
            "You're on a break since {}, with breaks at {}took {:02}:{:02}h.",
            start.with_timezone(&chrono::Local).format("%H:%M"),
            break_str,
            break_state.sum.num_hours(),
            break_state.sum.num_minutes() % 60
        );
        break_state.sum
    } else if break_state.sum > Duration::seconds(0) {
        println!(
            "You had breaks at {}with a total of {:02}:{:02}h.",
            break_str,
            break_state.sum.num_hours(),
            break_state.sum.num_minutes() % 60
        );
        break_state.sum
    } else {
        break_state.sum
    };

    if let Some(daily) = balance.config.as_ref().unwrap_or_default().daily_hours {
        let daily = Duration::hours(daily as i64);
        let remaining = daily - dur + pause;
        let daily_range = balance
            .daily_range(Local::now().date_naive(), Local)
            .unwrap() // TODO: get rid of unwrap
            .fold(Duration::seconds(0), |acc, (_, dur)| {
                log::trace!("dur: {:?}", dur);
                acc + dur.into()
            });
        log::trace!(
            "Previously worked hours {:?}, remaining: {:?}",
            daily_range,
            remaining
        );
        if remaining < Duration::zero() {
            println!(
                "You're done for today. You have {:02}:{:02}h overhours.",
                (-remaining).num_hours(),
                (-remaining).num_minutes() % 60
            );
        } else if !(remaining - daily).is_zero() {
            println!(
                "You still need to work {:02}:{:02}h.",
                remaining.num_hours(),
                remaining.num_minutes() % 60,
            );
        }
    }
    if let Some(hours) = balance.calculate_overhours() {
        println!(
            "You have total overhours of {:02}:{:02}h",
            hours.num_hours(),
            hours.num_minutes() % 60
        );
    }
}

fn avg_start_time(balance: &TimeBalance) -> Result<()> {
    if let Some(avg_time) = balance.avg_start_time() {
        let date = Utc::now().date_naive();
        let utc_time = NaiveDateTime::new(date, avg_time);
        let local_time = utc_time.and_local_timezone(Local).unwrap();
        println!("Average start time: {}", local_time.format("%H:%M:%S"));
    }
    Ok(())
}
