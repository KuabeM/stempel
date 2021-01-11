use crate::month::Month;
use crate::storage::{WorkStorage, WorkType};
use colored::*;
use failure::{bail, Error};
use log::{info, warn};
use std::path::Path;
use std::time::Duration;

use crate::commands::control::time_from_duration;

/// Prints a summary of the current storage either for one month or all data.
///
/// Handler for the `stats` sub command.
pub fn stats<P: AsRef<Path>>(storage: P, month: Option<Month>) -> Result<(), Error> {
    if !storage.as_ref().exists() {
        bail!(
            "There is no time storage {:?}, start working first. It creates the file if necessary",
            storage.as_ref()
        );
    }
    let store = WorkStorage::from_file(&storage)?;
    match month {
        Some(m) => monthly_stats(&storage, m),
        None => {
            if store.work_sets.len() < 6 {
                info!("{}", store.stats());
                Ok(())
            } else {
                all_monthly_stats(&storage)
            }
        }
    }
}

/// Prints all entry of all months in `storage`.
fn all_monthly_stats<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let store = WorkStorage::from_file(storage)?;
    let months = store.months();
    let weeks = store.weeks();
    info!("Here are your stats, {}:", store.name());
    for m in months {
        let work_per_m =
            store.filter(|w| Month::from(w.start.date().format("%B").to_string()) == m);
        println!(
            "{} {: >2}{}",
            "Month".green(),
            m.to_string().green(),
            ":".green()
        );
        for w in &weeks {
            let work_per_w: Duration = work_per_m
                .filter(|s| &s.start.date().format("%W").to_string() == w)
                .filter(|s| s.ty == WorkType::Work)
                .work_sets
                .iter()
                .fold(Duration::new(0, 0), |acc, d| acc + d.duration);
            if work_per_w.as_nanos() > 0 {
                let (h, min) = time_from_duration(work_per_w);
                println!(" Week {}: {: >4}:{:02}h", w, h, min);
            }
        }
    }
    if let Ok(s) = store.try_start() {
        println!(" {}", s);
    }
    if let Ok(b) = store.try_break() {
        println!(" {}", b);
    }
    Ok(())
}

/// Prints the entries in the `storage` for one `month`.
fn monthly_stats<P: AsRef<Path>>(storage: P, month: Month) -> Result<(), Error> {
    let store = WorkStorage::from_file(storage)?;
    let weeks = store.weeks();
    let work_per_m =
        store.filter(|w| Month::from(w.start.date().format("%B").to_string()) == month);
    if work_per_m.work_sets.is_empty() {
        warn!("{}, you did not work in {}!", store.name(), month);
        return Ok(());
    }

    info!("Here are your stats for {}, {}:", month, store.name());
    for w in &weeks {
        let work_per_w: Duration = work_per_m
            .filter(|s| &s.start.date().format("%W").to_string() == w)
            .work_sets
            .iter()
            .fold(Duration::new(0, 0), |acc, d| acc + d.duration);
        if work_per_w.as_nanos() > 0 {
            let (h, min) = time_from_duration(work_per_w);
            println!(" Week {}: {: >4}:{:02}h", w, h, min);
        }
    }
    if let Ok(s) = store.try_start() {
        println!(" {}", s);
    }
    if let Ok(b) = store.try_break() {
        println!(" {}", b);
    }
    Ok(())
}
