use chrono::prelude::*;
use chrono::Duration;

use failure::{bail, format_err, Error};
use serde::{Deserialize, Serialize};

use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::{
    collections::BTreeMap,
    io::{BufReader, Read, Write},
};

fn nanoseconds(dur: &Duration) -> i32 {
    0i32
}

use crate::storage::WorkStorage;

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
        dur.inner.clone()
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub(crate) struct DurationDef {
    #[serde(flatten)]
    #[serde(with = "ChronoDuration")]
    inner: Duration,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct TimeBalance {
    start: Option<DateTime<Utc>>,
    breaking: Option<DateTime<Utc>>,
    breaks: Vec<DurationDef>,
    #[serde(rename = "account")]
    time_account: BTreeMap<DateTime<Utc>, DurationDef>,
}

impl TimeBalance {
    fn new() -> Self {
        Self {
            time_account: BTreeMap::new(),
            start: None,
            breaking: None,
            breaks: Vec::new(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.start = None;
        self.breaks.clear();
    }

    pub(crate) fn cancel(&mut self) -> Result<(), Error> {
        match self.breaking {
            None => self
                .start
                .and_then(|_| {
                    self.start = None;
                    Some(())
                })
                .ok_or_else(|| format_err!("Nothing to cancel")),
            Some(_) => {
                self.breaking = None;
                Ok(())
            }
        }
    }

    pub(crate) fn start(&mut self, time: DateTime<Utc>) -> Result<(), DateTime<Utc>> {
        match self.start {
            None => {
                self.start = Some(time);
                Ok(())
            }
            Some(s) => Err(s),
        }
    }

    pub(crate) fn stop(&mut self, time: DateTime<Utc>) -> Result<Duration, Error> {
        let start = self
            .start
            .ok_or_else(|| format_err!("You did not start working"))?;
        let breaks = self.accumulate_breaks();
        let duration = time
            .signed_duration_since(start)
            .checked_sub(&breaks)
            .ok_or_else(|| format_err!("Your break was longer than your work"))?;
        self.insert(time, duration.into());
        self.reset();
        Ok(duration)
    }

    pub(crate) fn accumulate_breaks(&self) -> Duration {
        self.breaks
            .iter()
            .fold(Duration::seconds(0), |acc, b| acc + b.clone().into())
    }

    pub(crate) fn start_break(&mut self, time: DateTime<Utc>) -> Result<Duration, Error> {
        self.start
            .and_then(|s| {
                // TODO: check if there is a break already
                self.breaking = Some(time);
                let dur = time.signed_duration_since(s);
                Some(dur)
            })
            .ok_or_else(|| format_err!("You're not tracking your work so you can't take a break"))
    }

    pub(crate) fn finish_break(&mut self, time: DateTime<Utc>) -> Result<Duration, Error> {
        self.start
            .ok_or_else(|| format_err!("You can't break if you haven't started."))?;
        let break_start = self
            .breaking
            .ok_or_else(|| format_err!("You're not on a break right now."))?;

        let dur = time.signed_duration_since(break_start);
        self.breaks.push(dur.into());
        self.breaking = None;

        Ok(dur)
    }

    fn range(
        &self,
        lower: DateTime<Utc>,
        upper: DateTime<Utc>,
    ) -> impl Iterator<Item = (&DateTime<Utc>, &DurationDef)> {
        let range = lower..upper;
        self.time_account.range(range)
    }

    pub fn year_range(&self, year: u32) -> impl Iterator<Item = (&DateTime<Utc>, &DurationDef)> {
        let lower = Utc.ymd(year as i32, 1, 1).and_hms(0, 0, 0);
        let upper = Utc.ymd(year as i32, 12, 31).and_hms(23, 59, 59);
        self.range(lower, upper)
    }

    pub fn month_range(
        &self,
        year: i32,
        month: Month,
    ) -> impl Iterator<Item = (&DateTime<Utc>, &DurationDef)> {
        let days_in_m = if month.number_from_month() == 12 {
            Utc.ymd(year + 1, month.succ().number_from_month(), 1)
                .signed_duration_since(Utc.ymd(year, month.number_from_month(), 1))
                .num_days()
        } else {
            Utc.ymd(year, month.succ().number_from_month() , 1)
                .signed_duration_since(Utc.ymd(year, month.number_from_month(), 1))
                .num_days()
        };
        let lower = Utc.ymd(year, month.number_from_month(), 1).and_hms(0, 0, 0);
        let upper = Utc.ymd(year, month.number_from_month(), days_in_m as u32).and_hms(23, 59, 59);
        self.range(lower, upper)
    }

    pub fn group_by_week(&self) -> () {
        // impl Iterator<Item = (&DateTime<Utc>, &Set)> {
        unimplemented!();
    }

    pub(crate) fn insert(&mut self, dt: DateTime<Utc>, dur: DurationDef) {
        self.time_account.insert(dt, dur);
    }

    fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        serde_json::from_reader(reader)
            .map_err(|e| format_err!("Failed to deserialize json: {}", e))
    }

    fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        serde_json::to_writer(writer, &self)
            .map_err(|e| format_err!("Failed to serialize to json: {}", e))
    }

    pub fn from_file<P: AsRef<Path>>(path: P, create: bool) -> Result<Self, Error> {
        match File::open(&path) {
            Ok(f) => {
                let mut reader = BufReader::new(f);
                Self::from_reader(&mut reader)
            }
            Err(_) if create => Ok(TimeBalance::new()),
            Err(e) => bail!("Failed to open database: {}", e),
        }
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        match OpenOptions::new().write(true).truncate(true).open(&path) {
            Ok(mut f) => self.write(&mut f),
            Err(_) => {
                log::info!(
                    "Creating a new database {}",
                    path.as_ref().to_str().unwrap()
                );
                let mut f = File::create(&path)
                    .map_err(|e| format_err!("There is no database and creating failed: {}", e))?;
                self.write(&mut f)
            }
        }
    }
}

impl std::fmt::Display for TimeBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (s, d) in self.time_account.iter() {
            let local = s.with_timezone(&Local).format("%d/%m/%Y, %H:%M");
            let dur = Duration::from(d);
            write!(
                f,
                "{}: {}:{}h\n",
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
            time_account,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_file_works() {
        let naive = NaiveDate::from_ymd(2021, 01, 27).and_hms(14, 19, 21);
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
        let naive = NaiveDate::from_ymd(2021, 01, 27).and_hms(14, 19, 21);
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
