use log::info;
use std::path::Path;

use crate::error::TimeError;
use crate::storage::*;

pub fn start<P: AsRef<Path>>(storage: P) -> Result<(), TimeError> {
    let mut store = WorkStorage::from_file(storage)?;
    store.add_set(WorkSet::new(
        WorkType::Start,
        std::time::Duration::from_secs(1),
    ));

    info!("store: {:?}", store);
    Ok(())
}

pub fn stop<P: AsRef<Path>>(storage: P) -> Result<(), TimeError> {
    let mut store = WorkStorage::from_file(storage)?;
    store.add_set(WorkSet::new(
        WorkType::Work,
        std::time::Duration::from_secs(1),
    ));

    Ok(())
}

pub fn stats<P: AsRef<Path>>(storage: P) -> Result<(), TimeError> {
    let mut store = WorkStorage::from_file(storage)?;

    Ok(())
}
