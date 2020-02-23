use std::path::PathBuf;
use structopt::StructOpt;
#[macro_use]
use log::*;

mod error;
mod storage;
use error::TimeError;
use storage::*;

#[derive(StructOpt, Debug)]
#[structopt(about = "track the time spent with your fun colleagues")]
enum Opt {
    Start {
        /// Time when started
        time: String,
        /// Path to storage file
        #[structopt(short, long, default_value = "./time.json")]
        storage: PathBuf,
    },
    Stop {
        /// Time when started
        time: String,
        /// Path to storage file
        #[structopt(short, long, default_value = "./time.json")]
        storage: PathBuf,
    },
}

fn main() -> Result<(), TimeError> {
    env_logger::init();

    match Opt::from_args() {
        Opt::Start { time, storage } => println!("{:?}", storage),
        Opt::Stop { time, storage } => println!("{:?}", storage),
    }

    // let st = WorkStorage::from_file(opt.storage)?;

    Ok(())
}
