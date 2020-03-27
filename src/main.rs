use common_failures::quick_main;
use failure::{Error};
use log::{debug, info, warn};
use std::path::PathBuf;
use structopt::StructOpt;

mod commands;
mod storage;

#[derive(StructOpt, Debug)]
#[structopt(about = "track the time spent with your fun colleagues")]
enum Opt {
    Start {
        /// Time when started
        time: Option<String>,
        /// Path to storage file
        #[structopt(short, long, default_value = "./stempel.json")]
        storage: PathBuf,
    },
    Stop {
        /// Time when started
        time: Option<String>,
        /// Path to storage file
        #[structopt(short, long, default_value = "./stempel.json")]
        storage: PathBuf,
    },
    Stats {
        /// Path to storage file
        #[structopt(short, long, default_value = "./stempel.json")]
        storage: PathBuf,
    },
}

fn run() -> Result<(), Error> {
    env_logger::init();

    match Opt::from_args() {
        Opt::Start { time, storage } => {
            warn!("time: {:?}, store: {:?}", time, storage);
            commands::start(storage)?;
        }
        Opt::Stop { time, storage } => {
            info!("time: {:?}, store: {:?}", time, storage);
            commands::stop(storage)?;
        }
        Opt::Stats { storage } => {
            info!("Stats about {:?}", storage);
            commands::stats(storage)?;
        }
    }

    debug!("Finished this run");
    Ok(())
}

quick_main!(run);
