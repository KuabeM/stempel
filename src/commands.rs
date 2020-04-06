use chrono::{DateTime, Local};
use failure::{bail, Error};
use log::debug;
use std::path::Path;
use std::time::Duration;
use colored::*;

use crate::storage::*;

pub fn start<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let mut store = WorkStorage::from_file(&storage)?;
    let now = Duration::new(0, 0);
    let date: DateTime<Local> = Local::now();
    store.add_set(WorkSet::new(WorkType::Start, now, date));

    debug!("store: {:?}", store);
    store.write(&storage)?;
    Ok(())
}

pub fn stop<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    if !storage.as_ref().exists() {
        bail!(
            "There is no time storage {:?}, start working first. It creates the file if necessary",
            storage.as_ref()
        );
    }
    let mut store = WorkStorage::from_file(&storage)?;
    let start = store.try_start()?;
    let now: DateTime<Local> = Local::now();
    let duration: Duration = now.signed_duration_since(start).to_std()?;
    if duration > Duration::new(24 * 60 * 60, 0) {
        println!(
            "{} {}, you worked more than a day? It's been {}h",
            " WARN ".yellow(),
            store.name(),
            duration.as_secs() / 3600
        );
    }

    store.del_start();
    store.add_set(WorkSet::new(WorkType::Work, duration, start));
    store.write(&storage)?;
    println!(
        "{} You worked {}h today. Enjoy your evening :)",
        " INFO ".green(),
        duration.as_secs() / 3600
    );
    Ok(())
}

pub fn stats<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let store = WorkStorage::from_file(storage)?;
    println!("{}", store.stats());
    Ok(())
}

pub fn monthly_stats<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let store = WorkStorage::from_file(storage)?;
    let months = store.months();
    // TODO filter months
    Ok(())
}
