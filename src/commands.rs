use failure::Error;
use log::info;
use std::path::Path;

use crate::storage::*;

pub fn start<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let mut store = WorkStorage::from_file(&storage)?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

    store.add_set(WorkSet::new(WorkType::Start, now));

    info!("store: {:?}", store);
    store.write(&storage)?;
    Ok(())
}

pub fn stop<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let mut store = WorkStorage::from_file(&storage)?;
    let start = store.try_start()?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

    store.del_start();
    store.add_set(WorkSet::new(WorkType::Work, now - start));
    store.write(&storage)?;
    println!(
        "You worked {}s today. Enjoy your evening :)",
        (now - start).as_secs()
    );
    Ok(())
}

pub fn stats<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let store = WorkStorage::from_file(storage)?;
    println!("{}", store.stats());
    Ok(())
}
