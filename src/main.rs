use env_logger::Env;
use failure::Error;
use log::debug;
use std::path::PathBuf;
use structopt::StructOpt;

mod commands;
mod delta;
mod month;
mod storage;

use delta::{parse_time, OffsetTime};
use month::Month;

#[derive(StructOpt, Debug)]
#[structopt(about = "Track the time spent with your fun colleagues")]
enum Opt {
    /// Start a working period or a break
    Start(Action),
    /// Stop a working period or a break
    Stop(Action),
    /// Cancel the last action (Stop can't be undone)
    Cancel(Action),
    /// Print statistics about tracked time
    Stats {
        /// Path to storage file
        #[structopt(short, long)]
        storage: Option<PathBuf>,
        /// Month of which the stats are shown
        #[structopt(short, long)]
        month: Option<Month>,
    },
}

/// Options for structop subcommands
#[derive(StructOpt, Debug)]
struct Action {
    /// Start/Stop break
    #[structopt(subcommand)]
    breaking: Option<Break>,
    /// Time when started
    #[structopt(short, long, parse(try_from_str = parse_time))]
    offset: Option<OffsetTime>,
    /// Path to storage file
    #[structopt(short, long)]
    storage: Option<PathBuf>,
}

#[derive(StructOpt, Debug, PartialEq)]
enum Break {
    /// Manage a break
    #[structopt(name = "break")]
    Breaking,
}

fn run() -> Result<(), Error> {
    env_logger::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    let default_path = PathBuf::from(std::env::var("HOME")? + "/.config/stempel.json");

    match Opt::from_args() {
        Opt::Start(action) if action.breaking.is_some() => {
            let time = action.offset.unwrap_or_default().date_time;
            debug!("Start break {}, store {:?}", &time, action.storage);
            commands::start_break(action.storage.unwrap_or(default_path), time)?;
        }
        Opt::Stop(action) if action.breaking.is_some() => {
            let time = action.offset.unwrap_or_default().date_time;
            debug!("Stop break {}, store {:?}", time, action.storage);
            commands::stop_break(action.storage.unwrap_or(default_path), time)?;
        }
        Opt::Start(action) => {
            let time = action.offset.unwrap_or_default().date_time;
            debug!("Start at {}, store in {:?}", time, action.storage);
            commands::start(action.storage.unwrap_or(default_path), time)?;
        }
        Opt::Stop(action) => {
            let time = action.offset.unwrap_or_default().date_time;
            debug!("Stop at {:?}, store in {:?}", time, action.storage);
            commands::stop(action.storage.unwrap_or(default_path), time)?;
        }
        Opt::Cancel(action) => {
            debug!("Cancel");
            let breaking = action.breaking.is_some();
            commands::cancel(action.storage.unwrap_or(default_path), breaking)?;
        }
        Opt::Stats { storage, month } => {
            debug!("Stats of `{:?}`", storage);
            commands::stats(&storage.unwrap_or(default_path), month)?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        log::error!("{}", e.to_string());
        std::process::exit(1);
    }
}
