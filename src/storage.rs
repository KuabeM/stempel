use chrono::{DateTime, Utc, Local};
use failure::{bail, format_err, Error};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use crate::month::Month;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum WorkType {
    Work,
    Start,
    Stop,
    Break,
}

impl TryFrom<&str> for WorkType {
    type Error = Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        match input.to_lowercase().as_str() {
            "work" => Ok(WorkType::Work),
            "start" => Ok(WorkType::Start),
            "stop" => Ok(WorkType::Stop),
            "break" => Ok(WorkType::Break),
            _ => bail!("Failed to parse {} into WorkType", input),
        }
    }
}

impl fmt::Display for WorkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkType::Start => write!(f, "Start"),
            WorkType::Work => write!(f, " Work"),
            WorkType::Stop => write!(f, " Stop"),
            WorkType::Break => write!(f, "Break"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkSet {
    pub ty: WorkType,
    pub duration: Duration,
    pub start: DateTime<Utc>,
}

impl WorkSet {
    pub fn new(ty: WorkType, duration: Duration, start: DateTime<Utc>) -> Self {
        WorkSet {
            ty,
            duration,
            start,
        }
    }
}

impl fmt::Display for WorkSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dur = chrono::Duration::from_std(self.duration).map_err(|_| fmt::Error)?;
        let loc: DateTime<Local> = DateTime::from(self.start);
        write!(
            f,
            "{} on {}: {:>02}:{:>02} h",
            self.ty,
            loc.format("%d/%m/%Y, %H:%M (%a)"),
            dur.num_hours(),
            dur.num_minutes() - dur.num_hours() * 60
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkStorage {
    pub name: String,
    pub work_sets: Vec<WorkSet>,
}

impl WorkStorage {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        match File::open(path) {
            Ok(f) => {
                let reader = BufReader::new(f);
                serde_json::from_reader(reader)
                    .map_err(|e| format_err!("Failed to deserialize json: {}", e))
            }
            Err(_) => {
                println!("Enter your name: ");
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer)?;
                Ok(WorkStorage::new(buffer.trim_end().to_string()))
            }
        }
    }

    fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(&self).map_err(|e| format_err!("Failed to serialize to json: {}", e))
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        std::fs::write(path, self.to_json()?)?;
        Ok(())
    }

    fn new(name: String) -> Self {
        WorkStorage {
            name,
            work_sets: Vec::new(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn add_set(&mut self, set: WorkSet) {
        self.work_sets.push(set);
    }

    pub fn stats(&self) -> String {
        self.to_string()
    }

    pub fn try_start(&self) -> Result<WorkSet, Error> {
        let start = self
            .work_sets
            .iter()
            .find(|w| w.ty == WorkType::Start)
            .map(|w| w.clone());
        match start {
            Some(s) => Ok(s),
            None => bail!(
                "You want to stop but you never started, strange work ethics, {}",
                self.name
            ),
        }
    }

    pub fn try_break(&self) -> Result<WorkSet, Error> {
        let breaked = self
            .work_sets
            .iter()
            .rev()
            .find(|w| w.ty == WorkType::Break)
            .map(|w| w.clone());
        match breaked {
            Some(s) => Ok(s),
            None => Err(format_err!("You deserve that break")),
        }
    }

    pub fn del_start(&mut self) {
        self.work_sets.retain(|w| w.ty != WorkType::Start);
    }

    pub fn del_break(&mut self) {
        self.work_sets.retain(|w| w.ty != WorkType::Break);
    }

    pub fn months(&self) -> Vec<Month> {
        let mut months: Vec<Month> = self
            .work_sets
            .iter()
            .map(|m| Month::from(m.start.date().format("%B").to_string()))
            .collect();
        months.sort();
        months.dedup();
        months
    }

    pub fn weeks(&self) -> Vec<String> {
        let mut weeks: Vec<String> = self
            .work_sets
            .iter()
            .map(|m| m.start.date().format("%W").to_string())
            .collect();
        weeks.sort();
        weeks.dedup();
        weeks
    }

    pub fn filter<P>(&self, predicate: P) -> WorkStorage
    where
        P: Fn(&WorkSet) -> bool,
    {
        let work_sets: Vec<WorkSet> = self
            .work_sets
            .clone()
            .into_iter()
            .filter(|w| predicate(w))
            .collect();
        WorkStorage {
            name: self.name.clone(),
            work_sets,
        }
    }
}

impl fmt::Display for WorkStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.name)?;
        for set in self.work_sets.iter() {
            write!(f, "\n\t{}", set)?;
        }
        Ok(())
    }
}

#[test]
fn worktype_parses() {
    let w = "work";
    assert_eq!(WorkType::try_from(w).unwrap(), WorkType::Work);
    let w = "start";
    assert_eq!(WorkType::try_from(w).unwrap(), WorkType::Start);
    let w = "stop";
    assert_eq!(WorkType::try_from(w).unwrap(), WorkType::Stop);
    let w = "break";
    assert_eq!(WorkType::try_from(w).unwrap(), WorkType::Break);
    let w = "something";
    assert!(WorkType::try_from(w).is_err());
}

#[test]
fn serde_ok() {
    let store_raw = r#"{
        "name": "test",
        "work_sets": [
            {
                "ty": "Work",
                "duration": {"secs": 2, "nanos": 0},
                "start": "2020-03-27T10:22:12.755844511Z"
            },
            {
                "ty": "Start",
                "duration": {"secs": 2, "nanos": 0},
                "start": "2020-03-27T10:22:12.755844511Z"
            }
        ]
    }"#;
    let mut store: WorkStorage = serde_json::from_str(store_raw).expect("Failed to deserialize");
    assert_eq!(store.name, "test");
    assert_eq!(store.work_sets.first().unwrap().ty, WorkType::Work);
    assert_eq!(
        store.work_sets.first().unwrap().duration,
        std::time::Duration::from_secs(2)
    );
    assert_eq!(store.work_sets.len(), 2);

    let store_ser = serde_json::to_string(&store).expect("Failed to serialize");
    let dt: DateTime<Utc> = DateTime::parse_from_rfc3339("2020-03-27T10:22:12.755844511+00:00")
        .unwrap()
        .into();
    store.work_sets[0].start = dt;
    assert_eq!(store_raw.replace('\n', "").replace(' ', ""), store_ser);
    assert_eq!(store_ser, store.to_json().expect("Failed to serialize"));
}

#[test]
fn serde_throws() {
    let store_raw = r#"{
        "name": "test",
    }"#;
    assert!(serde_json::from_str::<WorkStorage>(store_raw).is_err());
}
