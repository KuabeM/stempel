//! Handler for the statistics subcommand.
//!
//! The main entry point is `stats` which then further decides what to do.

use crate::balance::{DurationDef, TimeBalance};
use crate::storage::{WorkStorage, WorkType};

use chrono::{DateTime, Datelike, Month, Utc};
use colored::*;
use failure::{bail, format_err, Error};
use itertools::Itertools;
use log::{info, warn};
use num_traits::FromPrimitive;

use std::convert::TryFrom;
use std::path::Path;

/// Prints a summary of the current storage either for one month.
///
/// Handler for the `stats` sub command.
pub fn stats<P: AsRef<Path>>(storage: P, month: Option<crate::month::Month>) -> Result<(), Error> {
    let year = Utc::now().year();
    if let Some(m) = month {
        let m = Month::from_u8(m as u8).ok_or_else(|| format_err!("Failed to parse month"))?;
        monthly_stats(&storage, year, m)?;
    } else {
        // let month = Month::from("now");
        let m = Month::from_u32(Utc::now().month())
            .ok_or_else(|| format_err!("Failed to parse current month"))?;
        monthly_stats(&storage, year, m.pred().pred())?;
        monthly_stats(&storage, year, m.pred())?;
        monthly_stats(&storage, year, m)?;
    }

    // let balance = TimeBalance::from_file(&storage, false)?;
    // let now = chrono::Utc::now().date();
    // let month = month.unwrap_or(Month::from("now"));
    // info!("Your balance for {}:", month);
    // for (s, d) in balance.month_range(year, month as u32) {
    //     let dur = chrono::Duration::from(d);
    //     println!(
    //         "{}: {:02}:{:02}h",
    //         s.format("%a, %d/%m"),
    //         dur.num_hours(),
    //         dur.num_minutes() % 60
    //     );
    // }

    // monthly_stats(&storage, year, Month::from("now"))?;

    Ok(())
}

/// Prints the entries in the `storage` for one `month` grouped by weeks.
fn monthly_stats<P: AsRef<Path>>(storage: P, year: i32, month: Month) -> Result<(), Error> {
    let balance = TimeBalance::from_file(&storage, false)?;
    let month_entries: Vec<(&DateTime<Utc>, &DurationDef)> =
        balance.month_range(year, month).collect();

    if month_entries.len() > 0 {
        println!("Account in {}:", month.name().green());
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
                "Week {}: {:02}:{:02}h",
                week,
                dur.num_hours(),
                dur.num_minutes() % 60
            );
        }
    }
    Ok(())
}
