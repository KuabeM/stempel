use crate::storage::WorkSet;
use crate::storage::{WorkStorage, WorkType};
use chrono::DateTime;
use chrono::{Local, Utc};
use failure::{bail, format_err, Error};
use log::{debug, info, warn};
use std::path::Path;
use std::time::Duration;

pub(crate) fn time_from_duration(dur: Duration) -> (u64, u64) {
    (dur.as_secs() / 3600, (dur.as_secs() / 60) % 60)
}

/// Handles the start of a working period and breaks called by subcommand
/// `start`.
///
/// `storage` points to the json storage file. Returns an error if there already
/// exists a start entry in the storage.
pub fn start<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<(), Error> {
    let mut store = WorkStorage::from_file(&storage)?;
    if let Ok(s) = store.try_start() {
        bail!(
            "You already started on {} at {}h",
            s.start.date().format("%d/%m/%Y"),
            s.start.time().format("%H:%M")
        );
    }

    let now = Duration::new(0, 0);
    store.add_set(WorkSet::new(WorkType::Start, now, time));

    info!(
        "Started at {}. Now be productive!",
        time.with_timezone(&Local).time().format("%H:%M")
    );
    debug!("store: {:?}", store);
    store.write(&storage)?;

    Ok(())
}

/// Calculates and writes the work to the storage based on a previous start.
///
/// `storage` points to the json storage file. Throws an error if there is no
/// such storage yet.
pub fn stop<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<(), Error> {
    if !storage.as_ref().exists() {
        bail!(
            "There is no time storage {:?}, start working first. It creates the file if necessary",
            storage.as_ref()
        );
    }
    let mut store = WorkStorage::from_file(&storage)?;
    let s = store.try_start()?;
    let duration: Duration = time.signed_duration_since(s.start).to_std()?;
    let (h, m) = time_from_duration(duration);
    if duration > Duration::new(24 * 60 * 60, 0) {
        warn!(
            "{}, you worked more than a day? It's been {}:{:02}h",
            store.name(),
            h,
            m
        );
    }
    // check if there is a break
    let break_dur = if let Ok(b) = store.try_break() {
        b.duration
    } else {
        Duration::new(0, 0)
    };
    let duration = duration - break_dur;

    store.delete_type(WorkType::Start);
    store.delete_type(WorkType::Break);
    store.add_set(WorkSet::new(WorkType::Work, duration, s.start));
    store.write(&storage)?;
    info!(
        "You worked {}:{:02}h today. Enjoy your evening \u{1F389}",
        h, m
    );
    Ok(())
}

/// Remove a previously started Break or Start entry from the storage.
///
/// Returns an Error if the specified type is not in the storage.
pub fn cancel<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let mut store = WorkStorage::from_file(&storage)?;
    if store.contains(WorkType::Break) {
        store.delete_type(WorkType::Break);
        store.write(storage)?;
        Ok(())
    } else if store.contains(WorkType::Start) {
        store.delete_type(WorkType::Start);
        store.write(storage)?;
        Ok(())
    } else {
        Err(format_err!(
            "There's neither a Start nor a Break, so nothing to cancel here..."
        ))
    }
}

/// Stop a 'break' by adding a `break` entry to the storage as handler of
/// `break` subcommand of `stop`.
///
/// `storage` is the json storage file. Throws an error if there is no start
/// entry in the storage.
pub fn stop_break<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<(), Error> {
    let mut store = WorkStorage::from_file(&storage)?;
    if store.try_start().is_err() {
        bail!("You want to take a break, but you didn't start yet");
    }

    match store.try_break() {
        Ok(b) if b.ty == WorkType::Break => {
            let duration: Duration = time.signed_duration_since(b.start).to_std()?;
            if b.duration != Duration::new(0, 0) {
                bail!("There is already a break, do you want to start another one?");
            }

            let (h, m) = time_from_duration(duration);
            if duration > Duration::new(8 * 60 * 60, 0) {
                warn!(
                    "{}, your break of {}:{}h is quite long. Did you fall asleep?",
                    store.name(),
                    h,
                    m
                );
            }
            store.delete_type(WorkType::Break);
            store.add_set(WorkSet::new(WorkType::Break, duration, time));
            store.write(&storage)?;
            info!("You had a break for {}:{:02}h.", h, m);
            Ok(())
        }
        Err(_) => {
            bail!("You tried to end a break, but you never started one.");
        }
        Ok(_) => {
            unreachable!("try_break returned a none break which is impossbile");
        }
    }
}

/// Start a 'break' by adding a `break` entry to the storage as handler of
/// `break` subcommand of `start`.
///
/// `storage` is the json storage file. Throws an error if there is no start
/// entry in the storage.
pub fn start_break<P: AsRef<Path>>(storage: P, time: DateTime<Utc>) -> Result<(), Error> {
    let mut store = WorkStorage::from_file(&storage)?;
    let start = if let Ok(s) = store.try_start() {
        s
    } else {
        bail!("You want to take a break, but you didn't start yet");
    };

    match store.try_break() {
        Ok(b) if b.ty == WorkType::Break => {
            if b.duration == Duration::new(0, 0) {
                bail!("You already started a break.");
            }

            debug!("There is a break, starting a new one.");
            let dur = Duration::new(0, 0);
            store.add_set(WorkSet::new(WorkType::Break, dur, time));
            let dur = time.signed_duration_since(start.start);
            let (h, m) = time_from_duration(dur.to_std()?);
            info!("Started a break after {}:{:02}h of work.", h, m);

            store.write(&storage)?;
            Ok(())
        }
        Err(_) => {
            let dur = Duration::new(0, 0);
            store.add_set(WorkSet::new(WorkType::Break, dur, time));
            let dur = time.signed_duration_since(start.start);
            let (h, m) = time_from_duration(dur.to_std()?);
            info!("Started a break after {}:{:02}h of work.", h, m);

            store.write(&storage)?;
            Ok(())
        }
        Ok(_) => {
            unreachable!("try_break returned a none break which is impossbile");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn calc_time_from_dur() {
        let dur = Duration::from_secs(12 * 60 * 60);
        assert_eq!(time_from_duration(dur), (12, 0));
        assert_eq!(
            time_from_duration(dur.checked_add(Duration::from_secs(35 * 60)).unwrap()),
            (12, 35)
        );
        assert_eq!(
            time_from_duration(dur.checked_add(Duration::from_secs(5 * 60)).unwrap()),
            (12, 5)
        );
        assert_eq!(
            time_from_duration(
                dur.checked_add(Duration::from_secs(30 * 60 + 10 * 60 * 60))
                    .unwrap()
            ),
            (22, 30)
        );
        assert_eq!(
            time_from_duration(dur.checked_add(Duration::from_secs(55 * 60)).unwrap()),
            (12, 55)
        );
    }
}
