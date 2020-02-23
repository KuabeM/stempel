use std::time::Duration;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::BufReader;

use serde::{Serialize, Deserialize};

use crate::error::*;

#[derive(Serialize, Deserialize, Debug)]
enum WorkType {
    Work,
    Start,
    Stop,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkSet {
    ty: WorkType,
    duration: Duration,
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
                serde_json::from_reader(reader).map_err(|e| TimeError::SerializationError(e.to_string()))
            }
            Err(_) => Ok(WorkStorage::new()),
        }
    }

    pub fn new() -> Self {
        WorkStorage { name: String::new(), work_sets: Vec::new() }
    }
}
