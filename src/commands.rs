use log::info;
use std::path::Path;

use crate::error::TimeError;
use crate::storage::*;

pub fn start<P: AsRef<Path>>(storage: P) -> Result<(), TimeError> {
    let mut store = WorkStorage::from_file(&storage)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| TimeError::SerializationError(e.to_string()))?;

    store.add_set(WorkSet::new(WorkType::Start, now));

    info!("store: {:?}", store);
    store.write(&storage)?;
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
    let store = WorkStorage::from_file(storage)?;
    println!("{}", store.stats());
    Ok(())
}
