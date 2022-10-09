//! Handler for the start, stop and break subcommands.

use crate::balance::TimeBalance;

use crate::errors::*;
use chrono::{DateTime, Local, Utc};
use colored::*;
use std::{convert::TryFrom, path::Path};

/// Handles the start of a working period and breaks called by subcommand
/// `start`.
///
/// `storage` points to the json storage file. Creates the database file if it
/// does not exist. Returns an error if there already exists a start entry in
/// the storage.
pub fn start<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<()> {
    let mut balance = TimeBalance::from_file(&storage, true)?;
    balance.start(time).map_err(|e| {
        usage_err!(
            "You already started at {}",
            e.with_timezone(&Local).time().format("%H:%M")
        )
    })?;
    println!(
        "You started at {}, let's go!",
        time.with_timezone(&Local)
            .time()
            .format("%H:%M")
            .to_string()
            .green()
    );
    balance.to_file(storage)?;

    Ok(())
}

/// Calculates and writes the work to the storage based on a previous start.
///
/// `storage` points to the json storage file. Throws an error if there is no
/// such storage yet.
pub fn stop<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<()> {
    let mut balance = TimeBalance::from_file(&storage, false)?;
    let duration = balance.stop(time)?;
    println!(
        "You worked {}:{:02}h today. Enjoy your evening \u{1F389}",
        duration.num_hours(),
        duration.num_minutes() % 60
    );
    balance.to_file(&storage)?;

    Ok(())
}

/// Cancels a break if present, otherwise the start or throws an error. Handler
/// of the `cancel` subcommand.
///
/// `storage` is the path pointing to the database file.
pub fn cancel<P: AsRef<Path>>(storage: P) -> Result<()> {
    let mut balance = TimeBalance::from_file(&storage, false)?;
    balance.cancel()?;
    balance.to_file(&storage)?;
    println!("Canceled last action.");
    Ok(())
}

/// Stop a 'break', calculates the duration and writes it to the database.
///
/// Handler of `break stop` subcommand. `storage` is the json storage file.
/// Throws an error if there is no stared break in the database.
pub fn stop_break<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<()> {
    let mut balance = TimeBalance::from_file(&storage, false)?;
    let dur = balance.finish_break(time)?;
    println!(
        "You had a break for {}:{:02}h. Way to go!",
        dur.num_hours(),
        dur.num_minutes() % 60
    );
    balance.to_file(&storage)?;
    Ok(())
}

/// Start a 'break' by adding a `break` entry to the database.
///
/// Handler of the `break start` subcommand. `storage` is the database file.
/// Throws an error if there is no start entry in the database.
pub fn start_break<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<()> {
    let mut balance = TimeBalance::from_file(&storage, false)?;
    let dur = balance.start_break(time)?;
    println!(
        "Started a break after working {}:{:02}h.",
        dur.num_hours(),
        dur.num_minutes() % 60
    );
    balance.to_file(storage)?;
    Ok(())
}

pub fn migrate<P: AsRef<Path>>(path: P) -> Result<()> {
    let storage = crate::storage::WorkStorage::from_file(&path)?;
    let balance = TimeBalance::try_from(&storage)?;
    balance.to_file(&path)?;
    let migrated_path: String = (path.as_ref().to_string_lossy() + ".bak").to_string();
    storage.write(std::path::PathBuf::from(migrated_path))?;
    Ok(())
}
