use common_failures::quick_main;
use failure::Error;
use log::debug;
use std::path::PathBuf;
use structopt::StructOpt;
use env_logger::Env;

mod commands;
mod storage;
mod month;

use month::Month;

#[derive(StructOpt, Debug)]
#[structopt(about = "track the time spent with your fun colleagues")]
enum Opt {
    Start {
        /// Time when started
        time: Option<String>,
        /// Path to storage file
        #[structopt(short, long)]
        storage: Option<PathBuf>,
    },
    Stop {
        /// Time when started
        time: Option<String>,
        /// Path to storage file
        #[structopt(short, long)]
        storage: Option<PathBuf>,
    },
    Stats {
        /// Path to storage file
        #[structopt(short, long)]
        storage: Option<PathBuf>,
        /// Month of which the stats are shown
        #[structopt(short, long)]
        month: Option<Month>,
    },
}

fn run() -> Result<(), Error> {
    env_logger::from_env(Env::default().default_filter_or("info")).format_timestamp(None)
        .format_module_path(false)
        .init();

    match Opt::from_args() {
        Opt::Start { time, storage } => {
            debug!("Start at {:?}, store in {:?}", time, storage);
            commands::start(storage.unwrap_or(PathBuf::from(std::env::var("HOME")? + "/.config/stempel.json")))?;
        }
        Opt::Stop { time, storage } => {
            debug!("Stop at {:?}, store in {:?}", time, storage);
            commands::stop(storage.unwrap_or(PathBuf::from(std::env::var("HOME")? + "/.config/stempel.json")))?;
        }
        Opt::Stats { storage, month } => {
            debug!("Stats of `{:?}`", storage);
            commands::stats(&storage.unwrap_or(PathBuf::from(std::env::var("HOME")? + "/.config/stempel.json")), month)?;
        }
    }

    Ok(())
}

quick_main!(run);
