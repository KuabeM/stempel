//! Model of time balance.
//!
//! Load, write and manipulate the working time balance.

use chrono::prelude::*;
use chrono::Duration;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::{
    collections::BTreeMap,
    io::{BufReader, Read, Write},
};

use crate::storage::WorkStorage;

fn nanoseconds(_dur: &Duration) -> i32 {
    0i32
}

/// Alias for chrono::Duration with serde support.
#[derive(Serialize, Deserialize)]
#[serde(remote = "Duration")]
struct ChronoDuration {
    #[serde(getter = "Duration::num_seconds")]
    secs: i64,
    #[serde(getter = "nanoseconds")]
    nanos: i32,
}

impl From<ChronoDuration> for Duration {
    fn from(def: ChronoDuration) -> Duration {
        Duration::seconds(def.secs)
    }
}

impl From<Duration> for DurationDef {
    fn from(dur: Duration) -> DurationDef {
        Self { inner: dur }
    }
}

impl From<&DurationDef> for Duration {
    fn from(dur: &DurationDef) -> Self {
        dur.inner
    }
}

impl From<DurationDef> for Duration {
    fn from(dur: DurationDef) -> Self {
        dur.inner
    }
}

impl ToString for DurationDef {
    fn to_string(&self) -> String {
        self.inner.to_string()
    }
}

/// Wrapper around chrono::Duration for serde support
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(crate) struct DurationDef {
    #[serde(flatten)]
    #[serde(with = "ChronoDuration")]
    inner: Duration,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    pub month_stats: u8,
    pub daily_hours: Option<u8>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            month_stats: 2,
            daily_hours: None,
        }
    }
}

impl Default for &Config {
    fn default() -> Self {
        &Config {
            month_stats: 2,
            daily_hours: None,
        }
    }
}

/// A storage for completed and started work sets as well as started and
/// completed breaks.
///
/// Completed work sets are stored in a hash map with entries
/// `(start, duration)`. If a break or work is running, the corresponding
/// options hold the respective start time.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct TimeBalance {
    start: Option<DateTime<Utc>>,
    breaking: Option<DateTime<Utc>>,
    breaks: Vec<(DateTime<Utc>, DurationDef)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Config>,
    #[serde(rename = "account")]
    time_account: BTreeMap<DateTime<Utc>, DurationDef>,
}

impl TimeBalance {
    fn new() -> Self {
        Self {
            time_account: BTreeMap::new(),
            start: None,
            breaking: None,
            config: None,
            breaks: Vec::new(),
        }
    }

    /// Clears starts and breaks to b ready for a new work day.
    pub(crate) fn reset(&mut self) {
        self.start = None;
        self.breaks.clear();
    }

    /// Remove a started break or a started work if no break exists.
    pub(crate) fn cancel(&mut self) -> Result<()> {
        match self.breaking {
            None => self
                .start
                .map(|_| {
                    self.start = None;
                })
                .ok_or_else(|| anyhow!("Nothing to cancel")),
            Some(_) => {
                self.breaking = None;
                Ok(())
            }
        }
    }

    /// Add a start time to balance.
    pub(crate) fn start(&mut self, time: DateTime<Utc>) -> Result<(), DateTime<Utc>> {
        match self.start {
            None => {
                self.start = Some(time);
                Ok(())
            }
            Some(s) => Err(s),
        }
    }

    /// Stop the started time, calculate the duration by resolving all breaks
    /// and the time since start.
    pub(crate) fn stop(&mut self, time: DateTime<Utc>) -> Result<Duration> {
        let start = self
            .start
            .ok_or_else(|| anyhow!("You did not start working"))?;
        if let Some(b) = self.breaking {
            anyhow::bail!(
                "You're on a break since {}, won't stop your current work.",
                b.time().format("%H:%M")
            );
        }
        let breaks = self.accumulate_breaks();
        let duration = time
            .signed_duration_since(start)
            .checked_sub(&breaks)
            .ok_or_else(|| anyhow!("Your break was longer than your work"))?;
        self.insert(time, duration.into());
        self.reset();
        Ok(duration)
    }

    /// Sum up duration of all finished breaks.
    pub(crate) fn accumulate_breaks(&self) -> Duration {
        self.breaks
            .iter()
            .fold(Duration::seconds(0), |acc, b| acc + b.clone().1.into())
    }

    /// Get all breaks.
    pub(crate) fn get_breaks(&self) -> Vec<(DateTime<Utc>, Duration)> {
        self.breaks.iter().map(|(s, d)| (*s, d.into())).collect()
    }

    /// Add `time` as start of break.
    pub(crate) fn start_break(&mut self, time: DateTime<Utc>) -> Result<Duration> {
        self.start
            .map(|s| {
                // TODO: check if there is a break already
                self.breaking = Some(time);
                time.signed_duration_since(s)
            })
            .ok_or_else(|| anyhow!("You're not tracking your work so you can't take a break"))
    }

    /// Calculate duration of current break.
    pub(crate) fn finish_break(&mut self, time: DateTime<Utc>) -> Result<Duration> {
        self.start
            .ok_or_else(|| anyhow!("You can't break if you haven't started."))?;
        let break_start = self
            .breaking
            .ok_or_else(|| anyhow!("You're not on a break right now."))?;

        let dur = time.signed_duration_since(break_start);
        self.breaks.push((break_start, dur.into()));
        self.breaking = None;

        Ok(dur)
    }

    /// Extract all entries in map between two time points.
    fn range(
        &self,
        lower: DateTime<Utc>,
        upper: DateTime<Utc>,
    ) -> impl Iterator<Item = (&DateTime<Utc>, &DurationDef)> {
        let range = lower..upper;
        self.time_account.range(range)
    }

    /// Extract all entries from within one month.
    pub fn month_range(
        &self,
        year: i32,
        month: Month,
    ) -> impl Iterator<Item = (&DateTime<Utc>, &DurationDef)> {
        log::trace!("Range for month {:?}", month);
        let days_in_m = if month.number_from_month() == 12 {
            Utc.ymd(year + 1, month.succ().number_from_month(), 1)
                .signed_duration_since(Utc.ymd(year, month.number_from_month(), 1))
                .num_days()
        } else {
            Utc.ymd(year, month.succ().number_from_month(), 1)
                .signed_duration_since(Utc.ymd(year, month.number_from_month(), 1))
                .num_days()
        };
        log::trace!("Days in month {:?}: {}", month, days_in_m);
        let lower = Utc.ymd(year, month.number_from_month(), 1).and_hms(0, 0, 0);
        let upper = Utc
            .ymd(year, month.number_from_month(), days_in_m as u32)
            .and_hms(23, 59, 59);
        log::trace!("Lower: {:?}, Upper: {:?}", lower, upper);
        self.range(lower, upper)
    }

    /// Extract all entries from one day.
    pub fn daily_range<T: chrono::offset::TimeZone>(
        &self,
        day: Date<T>,
    ) -> impl Iterator<Item = (&DateTime<Utc>, &DurationDef)> {
        log::trace!("Entries for {:?}", day);
        let start = day.and_hms(0, 0, 0).with_timezone(&Utc);
        let end = day.and_hms(23, 59, 59).with_timezone(&Utc);
        self.range(start, end)
    }

    /// Insert a start time and the corresponding duration into map.
    pub(crate) fn insert(&mut self, dt: DateTime<Utc>, dur: DurationDef) {
        self.time_account.insert(dt, dur);
    }

    /// Deserialize json buffer.
    fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        serde_json::from_reader(reader)
            .map_err(|e| anyhow!("Failed to deserialize json: {}. Try 'stempel migrate' to migrate to new json format.", e))
    }

    /// Serialize time balance to json.
    fn write<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        serde_json::to_writer(writer, &self)
            .map_err(|e| anyhow!("Failed to serialize to json: {}", e))
    }

    /// Read from json file.
    pub fn from_file<P: AsRef<Path>>(path: P, create: bool) -> Result<Self> {
        match File::open(&path) {
            Ok(f) => {
                let mut reader = BufReader::new(f);
                let s = Self::from_reader(&mut reader)?;
                Ok(s)
            }
            Err(_) if create => Ok(TimeBalance::new()),
            Err(e) => anyhow::bail!("Failed to open database: {}", e),
        }
    }

    /// Write time balance to json file.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        match OpenOptions::new().write(true).truncate(true).open(&path) {
            Ok(mut f) => self.write(&mut f),
            Err(_) => {
                log::info!(
                    "Creating a new database {}",
                    path.as_ref().to_str().unwrap()
                );
                let mut f = File::create(&path)
                    .map_err(|e| anyhow!("There is no database and creating failed: {}", e))?;
                self.write(&mut f)
            }
        }
    }

    /// Get start point and duration since then. None if there is no start entry.
    pub fn start_state(&self) -> Option<(Duration, DateTime<Utc>)> {
        if let Some(s) = self.start {
            let dur = Utc::now().signed_duration_since(s);
            Some((dur, s))
        } else {
            None
        }
    }

    /// Get start and duration of break if any
    pub fn break_state(&self) -> BreakeState {
        let break_sum = self.accumulate_breaks();
        if self.start.is_none() {
            return BreakeState {
                current: None,
                breaks: self.get_breaks(),
                sum: break_sum,
            };
        }
        let current = self.breaking;
        let sum = Utc::now()
            .signed_duration_since(current.unwrap_or_else(Utc::now))
            .checked_add(&break_sum)
            .unwrap_or(break_sum);
        BreakeState {
            current,
            breaks: self.get_breaks(),
            sum,
        }
    }
}

pub(crate) struct BreakeState {
    pub current: Option<DateTime<Utc>>,
    pub breaks: Vec<(DateTime<Utc>, Duration)>,
    pub sum: Duration,
}

impl std::fmt::Display for TimeBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (s, d) in self.time_account.iter() {
            let local = s.with_timezone(&Local).format("%d/%m/%Y, %H:%M");
            let dur = Duration::from(d);
            writeln!(
                f,
                "{}: {}:{}h",
                local,
                dur.num_hours(),
                dur.num_minutes() % 60
            )?;
        }
        Ok(())
    }
}

impl TryFrom<&WorkStorage> for TimeBalance {
    type Error = anyhow::Error;
    fn try_from(other: &WorkStorage) -> Result<Self, Self::Error> {
        let start = other.try_start().map(|s| s.start).ok();
        let breaking = other.try_break().map(|b| b.start).ok();
        let breaks = Vec::new();
        let time_account: BTreeMap<DateTime<Utc>, DurationDef> = other
            .work_sets
            .iter()
            .filter_map(|e| {
                if e.ty == crate::storage::WorkType::Work {
                    Some((e.start, Duration::from_std(e.duration).unwrap().into()))
                } else {
                    None
                }
            })
            .collect();

        Ok(Self {
            start,
            breaking,
            breaks,
            config: None,
            time_account,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_file_works() {
        let naive = NaiveDate::from_ymd(2021, 1, 27).and_hms(14, 19, 21);
        let utc_dt = DateTime::from_utc(naive, chrono::Utc);
        let dur: DurationDef = Duration::seconds(10).into();
        let input = r#"{"start":null,"breaking":null,"breaks":[],"account":{""#.to_string()
            + &utc_dt.to_rfc3339_opts(SecondsFormat::Secs, true)
            + r#"":{"secs":10,"nanos":0}}}"#;
        println!("{}", input);
        let balance = TimeBalance::from_reader(&mut input.as_bytes()).expect("Failed to serialize");

        let mut expected = TimeBalance::new();
        expected.insert(utc_dt, dur);
        assert_eq!(balance, expected);
    }

    #[test]
    fn to_json_works() {
        let mut balance = TimeBalance::new();
        let naive = NaiveDate::from_ymd(2021, 1, 27).and_hms(14, 19, 21);
        let utc_dt = DateTime::from_utc(naive, chrono::Utc);
        let dur = Duration::seconds(10).into();
        balance.insert(utc_dt, dur);

        let mut bytes: Vec<u8> = Vec::new();
        balance.write(&mut bytes).expect("serialize works");

        let json = std::str::from_utf8(&bytes).expect("Bytes represent a string.");
        println!("{}", json);
        let json_string = r#"{"start":null,"breaking":null,"breaks":[],"account":{""#.to_string()
            + &utc_dt.to_rfc3339_opts(SecondsFormat::Secs, true)
            + r#"":{"secs":10,"nanos":0}}}"#;
        assert_eq!(json, json_string);
    }

    #[test]
    fn cancel_break() {
        let mut balance = TimeBalance::new();
        assert!(balance.cancel().is_err());
        balance.start(Utc::now()).expect("Starting works");
        balance.start_break(Utc::now()).expect("break works");
        balance.cancel().expect("Cancel of break works");
        balance.cancel().expect("Cancel of start works");
        assert!(balance.cancel().is_err());
    }
}
