//! Handler for the statistics subcommand.
//!
//! The main entry point is `stats` which then further decides what to do.

use crate::balance::{BrakeState, DurationDef, TimeBalance};

use chrono::{DateTime, Datelike, Month, Utc};
use colored::*;
use failure::{format_err, Error};
use itertools::Itertools;
use log::info;
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
        // let month = Month::from("now");
        let m = Month::from_u32(Utc::now().month())
            .ok_or_else(|| format_err!("Failed to parse current month"))?;
        info!("Here are your stats for the last 2 months:",);
        monthly_stats(&balance, year, m.pred().pred());
        monthly_stats(&balance, year, m.pred());
        monthly_stats(&balance, year, m);
    }

    println!();
    show_state(&balance);

    Ok(())
}

/// Prints the entries in the `storage` for one `month` grouped by weeks.
fn monthly_stats(balance: &TimeBalance, year: i32, month: Month) {
    let month_entries: Vec<(&DateTime<Utc>, &DurationDef)> =
        balance.month_range(year, month).collect();

    if !month_entries.is_empty() {
        println!("{}:", month.name().green());
        let mut cur_w = 0;
        for (week, group) in &month_entries.into_iter().group_by(|e| {
            let week_num = e.0.iso_week().week();
            if week_num == cur_w {
                cur_w
            } else {
                cur_w = week_num;
                cur_w
            }
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
    if let Some((dur, start)) = balance.start_state() {
        println!(
            "Started at {}, worked {:02}:{:02}h since then.",
            start.with_timezone(&chrono::Local).format("%H:%M"),
            dur.num_hours(),
            dur.num_minutes() % 60
        );
    }
    match balance.break_state() {
        BrakeState::Started(d, s) => {
            println!(
                "You're on a break since {}, total break duration today is {:02}:{:02}h.",
                s.with_timezone(&chrono::Local).format("%H:%M"),
                d.num_hours(),
                d.num_minutes() % 60
            )
        }
        BrakeState::Finished(d) => println!(
            "Your breaks lasted {:02}:{:02}h.",
            d.num_hours(),
            d.num_minutes() % 60
        ),
        BrakeState::NotActive => {}
    }
}
