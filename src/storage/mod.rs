use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::*;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum WorkType {
    Work,
    Start,
    Stop,
}

impl TryFrom<&str> for WorkType {
    type Error = TimeError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        match input.to_lowercase().as_str() {
            "work" => Ok(WorkType::Work),
            "start" => Ok(WorkType::Start),
            "stop" => Ok(WorkType::Stop),
            _ => Err(TimeError::SerializationError(format!(
                "Unknown type {}",
                input
            ))),
        }
    }
}

impl fmt::Display for WorkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkType::Start => write!(f, "Start"),
            WorkType::Work => write!(f, " Work"),
            WorkType::Stop => write!(f, " Stop"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkSet {
    ty: WorkType,
    duration: Duration,
}

impl WorkSet {
    pub fn new(ty: WorkType, duration: Duration) -> Self {
        WorkSet { ty, duration }
    }
}

impl fmt::Display for WorkSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0:>10}: {1:>4}s", self.ty, self.duration.as_secs())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkStorage {
    name: String,
    work_sets: Vec<WorkSet>,
}

impl WorkStorage {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, TimeError> {
        match File::open(path) {
            Ok(f) => {
                let reader = BufReader::new(f);
                serde_json::from_reader(reader)
                    .map_err(|e| TimeError::SerializationError(e.to_string()))
            }
            Err(_) => Ok(WorkStorage::new()),
        }
    }

    fn to_json(&self) -> Result<String, TimeError> {
        serde_json::to_string(&self).map_err(|e| TimeError::SerializationError(e.to_string()))
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), TimeError> {
        std::fs::write(path, self.to_json()?).map_err(|e| TimeError::IoError(e.to_string()))?;
        Ok(())
    }

    fn new() -> Self {
        WorkStorage {
            name: String::new(),
            work_sets: Vec::new(),
        }
    }

    pub fn add_set(&mut self, set: WorkSet) {
        self.work_sets.push(set);
    }

    pub fn stats(&self) -> String {
        self.to_string()
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
    assert_eq!(WorkType::try_from(w), Ok(WorkType::Work));
    let w = "start";
    assert_eq!(WorkType::try_from(w), Ok(WorkType::Start));
    let w = "stop";
    assert_eq!(WorkType::try_from(w), Ok(WorkType::Stop));
    let w = "something";
    assert_eq!(
        WorkType::try_from(w),
        Err(TimeError::SerializationError(
            "Unknown type something".to_string()
        ))
    );
}

#[test]
fn serde_ok() {
    let store_raw = r#"{
        "name": "test",
        "work_sets": [
            {
                "ty": "Work",
                "duration": {"secs": 2, "nanos": 0}
            }
        ]
    }"#;
    let store: WorkStorage = serde_json::from_str(store_raw).expect("Failed to deserialize");
    assert_eq!(store.name, "test");
    assert_eq!(store.work_sets.first().unwrap().ty, WorkType::Work);
    assert_eq!(
        store.work_sets.first().unwrap().duration,
        std::time::Duration::from_secs(2)
    );

    let store_ser = serde_json::to_string(&store).expect("Failed to serialize");
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
