//! Model of time balance.
//!
//! Load, write and manipulate the working time balance.

use chrono::prelude::*;
use chrono::Duration;

use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::ops::Add;
use std::ops::AddAssign;
use std::path::Path;
use std::{
    collections::BTreeMap,
    io::{BufReader, Read, Write},
};

use crate::cli_input::YesNo;
use crate::errors::*;

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

/// Wrapper around chrono::Duration for serde support
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub(crate) struct DurationDef {
    #[serde(flatten)]
    #[serde(with = "ChronoDuration")]
    inner: Duration,
}

impl DurationDef {
    pub fn zero() -> Self {
        Self {
            inner: Duration::zero(),
        }
    }
}

impl AsRef<Duration> for DurationDef {
    fn as_ref(&self) -> &Duration {
        &self.inner
    }
}

impl Display for DurationDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}h",
            self.inner.num_hours(),
            self.inner.num_minutes() % 60
        )
    }
}

impl Add for DurationDef {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let dur = self.inner + rhs.into();
        dur.into()
    }
}

impl AddAssign for DurationDef {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    pub month_stats: u8,
    pub daily_hours: Option<u8>,
    pub weekly_stats: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            month_stats: 2,
            daily_hours: None,
            weekly_stats: None,
        }
    }
}

impl Default for &Config {
    fn default() -> Self {
        &Config {
            month_stats: 2,
            daily_hours: None,
            weekly_stats: None,
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
                .ok_or_else(|| eyre!(usage_err!("Nothing to cancel"))),
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
            .ok_or_else(|| usage_err!("You did not start working"))?;
        if let Some(b) = self.breaking {
            bail!(usage_err!(
                "You're on a break since {}, won't stop your current work.",
                b.time().format("%H:%M")
            ));
        }
        let breaks = self.accumulate_breaks();
        let stop = if start.naive_local().date() != time.naive_local().date() {
            println!(
                "You started working on {}, do you really want to stop today? [y/N]",
                start.format("%d.%m.")
            );
            match YesNo::wait_for_decision()? {
                YesNo::Yes => time,
                YesNo::No => {
                    let time_stamp = time.time();
                    let date = start.naive_utc().date();
                    let stop = date.and_time(time_stamp);
                    stop.and_utc()
                }
            }
        } else {
            time
        };
        let duration = stop
            .signed_duration_since(start)
            .checked_sub(&breaks)
            .ok_or_else(|| usage_err!("Your break was longer than your work"))?;
        self.insert(stop, duration.into());
        self.reset();
        Ok(duration)
    }

    /// Sum up duration of all finished breaks.
    pub(crate) fn accumulate_breaks(&self) -> Duration {
        self.breaks
            .iter()
            .fold(Duration::seconds(0), |acc, b| acc + b.1.into())
    }

    /// Get all breaks.
    pub(crate) fn get_breaks(&self) -> Vec<(DateTime<Utc>, Duration)> {
        self.breaks.iter().map(|(s, d)| (*s, d.into())).collect()
    }

    /// Add `time` as start of break.
    pub(crate) fn start_break(&mut self, time: DateTime<Utc>) -> Result<Duration> {
        self.start
            .ok_or_else(|| {
                eyre!(usage_err!(
                    "You're not tracking your work so you can't take a break"
                ))
            })
            .map(|s| {
                if self.breaking.is_none() {
                    self.breaking = Some(time);
                    Some(time.signed_duration_since(s))
                } else {
                    None
                }
            })?
            .ok_or_else(|| eyre!(usage_err!("You're already on a break")))
    }

    /// Calculate duration of current break.
    pub(crate) fn finish_break(&mut self, time: DateTime<Utc>) -> Result<Duration> {
        self.start
            .ok_or_else(|| usage_err!("You can't break if you haven't started."))?;
        let break_start = self
            .breaking
            .ok_or_else(|| usage_err!("You're not on a break right now."))?;

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
        log::trace!("{:?} in {:?}", &range, &self.time_account);
        self.time_account.range(range)
    }

    /// Extract all entries from within one month.
    pub fn month_range(
        &self,
        year: i32,
        month: Month,
    ) -> Result<impl Iterator<Item = (&DateTime<Utc>, &DurationDef)>> {
        log::trace!("Range for month {:?}", month);
        let current = Utc
            .with_ymd_and_hms(year, month.number_from_month(), 1, 0, 0, 0)
            .latest()
            .ok_or(eyre!("Could not construct range"))?;
        let days_in_m = if month.number_from_month() == 12 {
            Utc.with_ymd_and_hms(year + 1, month.succ().number_from_month(), 1, 0, 0, 0)
                .earliest()
                .ok_or(eyre!("Could not construct range"))?
                .signed_duration_since(current)
                .num_days()
        } else {
            Utc.with_ymd_and_hms(year, month.succ().number_from_month(), 1, 0, 0, 0)
                .earliest()
                .ok_or(eyre!("Could not construct range"))?
                .signed_duration_since(current)
                .num_days()
        };
        log::trace!("Days in month {:?}: {}", month, days_in_m);
        let lower = Utc
            .with_ymd_and_hms(year, month.number_from_month(), 1, 0, 0, 0)
            .earliest()
            .ok_or(eyre!("Could not create range"))?;
        let upper = Utc
            .with_ymd_and_hms(
                year,
                month.number_from_month(),
                days_in_m as u32,
                23,
                59,
                59,
            )
            .latest()
            .ok_or(eyre!("Could not create range"))?;
        log::trace!("Lower: {:?}, Upper: {:?}", lower, upper);
        Ok(self.range(lower, upper))
    }

    /// Extract all entries from one day.
    pub fn daily_range<T: chrono::offset::TimeZone>(
        &self,
        day: NaiveDate,
        tz: T,
    ) -> Result<impl Iterator<Item = (&DateTime<Utc>, &DurationDef)>> {
        log::trace!("Entries for {:?}", day);
        let start = day
            .and_hms_opt(0, 0, 0)
            .ok_or(eyre!("Could not construct range"))?
            .and_local_timezone(tz.clone())
            .earliest()
            .ok_or(eyre!("Could not construct range"))?
            .with_timezone(&Utc);
        let end = day
            .and_hms_opt(23, 59, 59)
            .ok_or(eyre!("Could not construct range"))?
            .and_local_timezone(tz)
            .latest()
            .ok_or(eyre!("Could not construct range"))?
            .with_timezone(&Utc);
        Ok(self.range(start, end))
    }

    /// Extract all entries from the week of `date`.
    pub fn week_entries(
        &self,
        day: NaiveDate,
    ) -> impl Iterator<Item = (&DateTime<Utc>, &DurationDef)> {
        log::trace!("Entries in week of {:?}", day);
        let week = day.iso_week().week();
        self.time_account
            .iter()
            .filter(move |(d, _)| d.iso_week().week() == week)
    }

    /// Insert a start time and the corresponding duration into map.
    pub(crate) fn insert(&mut self, dt: DateTime<Utc>, dur: DurationDef) {
        self.time_account.insert(dt, dur);
    }

    /// Deserialize json buffer.
    fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        serde_json::from_reader(reader).wrap_err(
            "Failed to deserialize json. Try 'stempel migrate' to migrate to new json format",
        )
    }

    /// Serialize time balance to json.
    fn write<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        serde_json::to_writer(writer, &self).wrap_err("Failed to serialize to json")
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
            Err(e) => Err(e)
                .wrap_err_with(|| format!("Failed to open storage '{}'", path.as_ref().display())),
        }
    }

    /// Write time balance to json file.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        match OpenOptions::new().write(true).truncate(true).open(&path) {
            Ok(mut f) => self.write(&mut f),
            Err(_) => {
                log::info!("Creating a new storage file {}", path.as_ref().display());
                let mut f = File::create(&path).wrap_err_with(|| {
                    format!(
                        "There is no storage '{}' and creating failed",
                        path.as_ref().display()
                    )
                })?;
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

    /// Merge entries of the same day in the storage.
    pub fn canocicalize(&mut self) -> Result<()> {
        let mut current = self.time_account.iter();
        let mut peek = current.clone().skip(1).peekable();
        let mut merge = Vec::new();
        while let (Some(ne), Some(cur)) = (peek.peek(), current.next()) {
            if ne.0.date_naive() == cur.0.date_naive() {
                merge.push((*cur.0, *ne.0));
            }
            peek.next();
        }

        log::trace!("Removing keys {}: {:?}", merge.len(), merge);
        for (del_k, mer_k) in merge {
            let added = self
                .time_account
                .remove(&del_k)
                .ok_or(eyre!("Failed to remove duplicate element"))?;
            let cur = self
                .time_account
                .get(&mer_k)
                .ok_or(eyre!("Failed to update element"))?;
            log::trace!("Adding {:?} to {:?}", added, cur);
            *self
                .time_account
                .get_mut(&mer_k)
                .ok_or(eyre!("Failed to canocicalize"))? = *cur + added;
        }

        Ok(())
    }

    /// Calculate total overhours.
    pub fn calculate_overhours(&self) -> Option<Duration> {
        if let Some(daily) = self.config.as_ref().unwrap_or_default().daily_hours {
            let daily = Duration::hours(daily as i64);
            let hours = self
                .time_account
                .iter()
                .fold(Duration::zero(), |mut acc, (_, v)| {
                    let dur: Duration = v.into();
                    acc = acc + dur - daily;
                    acc
                });
            Some(hours)
        } else {
            None
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
    type Error = Error;
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
    use crate::storage::WorkSet;

    use super::*;

    #[test]
    fn from_file_works() {
        let naive = NaiveDate::from_ymd_opt(2021, 1, 27)
            .unwrap()
            .and_hms_opt(14, 19, 21)
            .unwrap();
        let utc_dt = DateTime::from_naive_utc_and_offset(naive, chrono::Utc);
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
        let naive = NaiveDate::from_ymd_opt(2021, 1, 27)
            .unwrap()
            .and_hms_opt(14, 19, 21)
            .unwrap();
        let utc_dt = DateTime::from_naive_utc_and_offset(naive, chrono::Utc);
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

    #[test]
    fn daily_range() {
        let mut balance = TimeBalance::new();
        let range = balance
            .daily_range(Utc::now().date_naive(), Utc)
            .expect("range works");
        assert!(range.last().is_none());
        {
            let start = Utc::now();
            balance
                .start(start - Duration::seconds(5))
                .expect("starting works");
            balance.stop(start).expect("stopping works");
            let range: Vec<(&DateTime<Utc>, &DurationDef)> = balance
                .daily_range(Utc::now().date_naive(), Utc)
                .expect("range works")
                .collect();
            assert_eq!(range.len(), 1);
            assert_eq!(
                *range.first().expect("has length 1"),
                (&start, &Duration::seconds(5).into())
            );
        }

        {
            let start = Utc::now()
                .date_naive()
                .and_hms_opt(20, 55, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .earliest()
                .unwrap();
            balance.start(start).expect("Starting works");
            let stop = start
                .checked_add_signed(Duration::minutes(90))
                .expect("adding works");
            balance.stop(stop).expect("stopping works");
            let range: Vec<(&DateTime<Utc>, &DurationDef)> = balance
                .daily_range(Utc::now().date_naive(), Utc)
                .expect("range works")
                .collect();
            assert_eq!(dbg!(&range).len(), 2);
            assert_eq!(
                *range.get(1).expect("has length 2"),
                (&stop, &Duration::minutes(90).into())
            );
        }
    }

    #[test]
    fn stringify() {
        let dur = Duration::nanoseconds(10)
            .checked_add(&Duration::seconds(1))
            .expect("adding works");
        let durdef = DurationDef::from(dur);
        assert_eq!(
            durdef.to_string(),
            format!("{:02}:{:02}h", dur.num_hours(), dur.num_minutes() % 60)
        );

        let dur_back = Duration::from(&durdef);
        assert_eq!(dur_back, durdef.into());
    }

    #[test]
    fn migrate() {
        let time = Utc::now();
        let start = WorkSet {
            ty: crate::storage::WorkType::Start,
            duration: std::time::Duration::from_secs(2),
            start: time,
        };
        let br = WorkSet {
            ty: crate::storage::WorkType::Break,
            duration: std::time::Duration::from_secs(1),
            start: time,
        };
        let work = WorkSet {
            ty: crate::storage::WorkType::Work,
            duration: std::time::Duration::from_secs(100),
            start: time,
        };
        let storage = WorkStorage {
            name: "test".to_string(),
            work_sets: vec![start, br, work],
        };

        let balance: TimeBalance = TimeBalance::try_from(&storage).expect("Conversion works");
        println!("{}", balance);
        assert_eq!(balance.start, Some(time));
        assert_eq!(balance.breaking, Some(time));
    }

    fn add_times(balance: &mut TimeBalance, dt: DateTime<Utc>, dur: i64) {
        balance
            .start(dt - Duration::minutes(dur))
            .expect("starting works");
        balance.stop(dt).expect("stopping works");
    }

    #[test]
    fn canocicalize_works() {
        let mut balance = TimeBalance::new();
        let now = Utc.with_ymd_and_hms(2022, 1, 12, 1, 20, 30).unwrap();
        add_times(&mut balance, now, 10);
        add_times(&mut balance, now + Duration::seconds(10), 12);
        add_times(&mut balance, now + Duration::seconds(20), 14);
        add_times(&mut balance, now + Duration::seconds(30), 18);
        assert_eq!(balance.time_account.len(), 4);
        balance.canocicalize().expect("Works");
        assert_eq!(balance.time_account.len(), 1);
        let sum = balance
            .time_account
            .iter()
            .fold(Duration::zero(), |mut acc, (_, v)| {
                acc = acc.checked_add(&v.into()).unwrap();
                acc
            });
        assert_eq!(sum, Duration::minutes(54));
    }

    #[test]
    fn overhours_work() {
        let mut balance = TimeBalance::new();
        balance.config = Some(Config {
            daily_hours: Some(1),
            ..Default::default()
        });

        let now = Utc.with_ymd_and_hms(2022, 1, 12, 1, 20, 30).unwrap();
        add_times(&mut balance, now, 70);
        log::trace!("balance: {:?}", balance);
        let overhours = balance.calculate_overhours();
        assert_eq!(overhours, Some(Duration::minutes(10)));

        add_times(&mut balance, now + Duration::seconds(10), 12);
        balance.canocicalize().expect("canocicalize works");
        let overhours = balance.calculate_overhours();
        assert_eq!(overhours, Some(Duration::minutes(22)));

        add_times(&mut balance, now - Duration::days(20), 64);
        let overhours = balance.calculate_overhours();
        assert_eq!(overhours, Some(Duration::minutes(26)));

        add_times(&mut balance, now + Duration::days(30), 58);
        let overhours = balance.calculate_overhours();
        assert_eq!(overhours, Some(Duration::minutes(24)));
    }
}
