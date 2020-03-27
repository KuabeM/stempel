use chrono::{DateTime, Local};
use failure::{Error, bail};
use log::{info, warn};
use std::path::Path;
use std::time::Duration;

use crate::storage::*;

pub fn start<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let mut store = WorkStorage::from_file(&storage)?;
    let now = Duration::new(0, 0);
    let date: DateTime<Local> = Local::now();
    store.add_set(WorkSet::new(WorkType::Start, now, date));

    info!("store: {:?}", store);
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
    if duration > Duration::new(26 * 60 * 60, 0) {
        warn!(
            "{}, you worked more than a day? It's been {}s",
            store.name(),
            duration.as_secs()
        );
    }

    store.del_start();
    store.add_set(WorkSet::new(WorkType::Work, duration, now));
    store.write(&storage)?;
    println!(
        "You worked {}s today. Enjoy your evening :)",
        duration.as_secs()
    );
    Ok(())
}

pub fn stats<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let store = WorkStorage::from_file(storage)?;
    println!("{}", store.stats());
    Ok(())
}
