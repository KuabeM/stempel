use env_logger::Env;
use failure::Error;
use log::debug;
use std::path::PathBuf;
use structopt::StructOpt;

mod commands;
mod month;
mod storage;

use month::Month;

#[derive(StructOpt, Debug)]
#[structopt(about = "Track the time spent with your fun colleagues")]
enum Opt {
    /// Start a working period or a break
    Start(Action),
    /// Stop a working period or a break
    Stop(Action),
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

#[derive(StructOpt, Debug)]
struct Action {
    /// Path to storage file
    #[structopt(short, long)]
    storage: Option<PathBuf>,
    /// Time when started
    #[structopt(short, long)]
    time: Option<String>,
    /// Start/Stop break
    #[structopt(subcommand)]
    breaking: Option<Break>,
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
            debug!("Start break {:?}, store {:?}", action.time, action.storage);
            commands::start_break(action.storage.unwrap_or(default_path))?;
        }
        Opt::Stop(action) if action.breaking.is_some() => {
            debug!("Stop break {:?}, store {:?}", action.time, action.storage);
            commands::stop_break(action.storage.unwrap_or(default_path))?;
        }
        Opt::Start(action) => {
            debug!("Start at {:?}, store in {:?}", action.time, action.storage);
            commands::start(action.storage.unwrap_or(default_path))?;
        }
        Opt::Stop(action) => {
            debug!("Stop at {:?}, store in {:?}", action.time, action.storage);
            commands::stop(action.storage.unwrap_or(default_path))?;
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
