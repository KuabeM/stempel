//! Handler for the start, stop and break subcommands.

use crate::balance::TimeBalance;

use chrono::{DateTime, Local, Utc};
use failure::{format_err, Error};
use log::info;
use std::{convert::TryFrom, path::Path};

/// Handles the start of a working period and breaks called by subcommand
/// `start`.
///
/// `storage` points to the json storage file. Creates the database file if it
/// does not exist. Returns an error if there already exists a start entry in
/// the storage.
pub fn start<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<(), Error> {
    let mut balance = TimeBalance::from_file(&storage, true)?;
    balance.start(time).map_err(|e| {
        format_err!(
            "You already started at {}",
            e.with_timezone(&Local).time().format("%H:%M")
        )
    })?;
    info!(
        "You started at {}, let's go!",
        time.with_timezone(&Local).time().format("%H:%M")
    );
    balance.to_file(storage)?;

    Ok(())
}

/// Calculates and writes the work to the storage based on a previous start.
///
/// `storage` points to the json storage file. Throws an error if there is no
/// such storage yet.
pub fn stop<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<(), Error> {
    let mut balance = TimeBalance::from_file(&storage, false)
        .map_err(|e| format_err!("There is no database: {}", e))?;
    let duration = balance.stop(time)?;
    info!(
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
pub fn cancel<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let mut balance = TimeBalance::from_file(&storage, false)?;
    balance.cancel()?;
    balance.to_file(&storage)?;
    info!("Canceled last action.");
    Ok(())
}

/// Stop a 'break', calculates the duration and writes it to the database.
///
/// Handler of `break stop` subcommand. `storage` is the json storage file.
/// Throws an error if there is no stared break in the database.
pub fn stop_break<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<(), Error> {
    let mut balance = TimeBalance::from_file(&storage, false)?;
    let dur = balance.finish_break(time)?;
    info!(
        "You had a break for {}:{}h. Way to go!",
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
pub fn start_break<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<(), Error> {
    let mut balance = TimeBalance::from_file(&storage, false)?;
    let dur = balance.start_break(time)?;
    info!(
        "Started a break at {}:{}",
        dur.num_hours(),
        dur.num_minutes() % 60
    );
    balance.to_file(storage)?;
    Ok(())
}

pub fn migrate<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let storage = crate::storage::WorkStorage::from_file(&path)?;
    let balance = TimeBalance::try_from(&storage)?;
    balance.to_file(&path)?;
    let migrated_path: String = (path.as_ref().to_string_lossy() + ".bak").to_string();
    storage.write(std::path::PathBuf::from(migrated_path))?;
    Ok(())
}
