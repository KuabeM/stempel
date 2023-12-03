use std::path::PathBuf;

use chrono::{DateTime, Utc};
pub use clap::Parser;
use clap::{Args, Subcommand};
use stempel::{
    delta::{parse_duration, parse_offset, parse_time},
    month::Month,
};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Path to storage file.
    #[arg(short, long)]
    pub storage: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start a working period.
    Start(Timings),
    /// Stop a working period.
    Stop(Timings),
    /// Start or stop a break.
    #[command(subcommand)]
    Break(StartStop),
    /// Cancel the last action (Stop can't be undone).
    Cancel,
    /// Print statistics about tracked time.
    Stats {
        /// Month of which the stats are shown.
        month: Option<Month>,
    },
    /// Migrate json storage from old to new format, creates backup file `*.bak` overwriting the
    /// original.
    Migrate,
    /// Configure how stempel displays things.
    Configure,
    /// Print shell completions.
    Completions {
        #[clap(long)]
        /// Provide the `shell` for which to generate the completion script.
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand, Debug)]
pub enum StartStop {
    /// Start a break, either now or based on flags.
    Start(Timings),
    /// Stop a break, either now or based on flags.
    Stop(Timings),
    /// A duration of a break in format `HH:MM`.
    #[command(alias = "dur")]
    Duration {
        #[arg(value_parser = parse_duration)]
        dur: chrono::Duration,
    },
}

#[derive(Debug, Args, Clone)]
pub struct Timings {
    /// Offset to current time in format `XX[h|m|s][+-]`.
    #[arg(short, long, conflicts_with = "time", value_parser = parse_offset, default_value = "0s+")]
    offset: DateTime<Utc>,
    /// An actual timepoint for starting or stopping an action in format `HH:MM`
    #[arg(short, long, conflicts_with = "offset", value_parser = parse_time)]
    time: Option<DateTime<Utc>>,
}

#[derive(Debug, Args, Clone)]
pub struct BreakTypes {
    /// Offset to current time in format `XX[h|m|s][+-]`.
    #[arg(short, long, conflicts_with = "time", value_parser = parse_offset, default_value = "0s+")]
    offset: DateTime<Utc>,
    /// An actual timepoint for starting or stopping an action in format `HH:MM`.
    #[arg(short, long, conflicts_with = "offset", value_parser = parse_time)]
    time: Option<DateTime<Utc>>,
}

impl Timings {
    pub fn time(&self) -> DateTime<Utc> {
        self.time.unwrap_or(self.offset)
    }
}
