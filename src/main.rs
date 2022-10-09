use env_logger::Env;
use log::debug;
use std::path::PathBuf;
use structopt::StructOpt;

use stempel::commands;
use stempel::delta::{parse_time, OffsetTime};
use stempel::errors::UsageError;
use stempel::month::Month;

#[derive(StructOpt, Debug)]
#[structopt(about = "Track the time spent with your fun colleagues")]
enum Opt {
    /// Start a working period.
    Start(Action),
    /// Stop a working period.
    Stop(Action),
    /// Start or stop a break.
    Break(StartStop),
    /// Cancel the last action (Stop can't be undone).
    Cancel(OptPath),
    /// Print statistics about tracked time.
    Stats {
        /// Path to storage file.
        #[structopt(short, long)]
        storage: Option<PathBuf>,
        /// Month of which the stats are shown.
        #[structopt(short, long)]
        month: Option<Month>,
    },
    /// Migrate json storage from old to new format, creates backup file
    /// `*.bak` overwriting the original.
    Migrate(OptPath),
    /// Configure internals of stempel.
    Configure(OptPath),
}

/// Subcommands for break subcommand.
#[derive(StructOpt, Debug, PartialEq)]
enum StartStop {
    /// Start a break
    Start(Action),
    /// Stop a break,
    Stop(Action),
}

/// Options for subcommands.
#[derive(StructOpt, Debug, PartialEq)]
struct Action {
    /// Offset to current time in format `XX[h|m|s][+-]`.
    #[structopt(short, long, parse(try_from_str = parse_time))]
    offset: Option<OffsetTime>,
    /// Path to storage file.
    #[structopt(short, long)]
    storage: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
struct OptPath {
    path: Option<PathBuf>,
}

fn run() -> color_eyre::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();
    color_eyre::install()?;

    let default_path = PathBuf::from(std::env::var("HOME")? + "/.config/stempel.json");

    match Opt::from_args() {
        Opt::Start(action) => {
            let time = action.offset.unwrap_or_default().date_time;
            debug!("Start at {}, store in {:?}", time, action.storage);
            commands::control::start(action.storage.unwrap_or(default_path), time)?;
        }
        Opt::Stop(action) => {
            let time = action.offset.unwrap_or_default().date_time;
            debug!("Stop at {:?}, store in {:?}", time, action.storage);
            commands::control::stop(action.storage.unwrap_or(default_path), time)?;
        }
        Opt::Break(startstop) => {
            let (is_start, action) = match startstop {
                StartStop::Start(action) => (true, action),
                StartStop::Stop(action) => (false, action),
            };
            let time = action.offset.unwrap_or_default().date_time;
            let storage = action.storage.unwrap_or(default_path);
            debug!("Break at {}, store in {:?}", time, storage);
            match is_start {
                true => commands::control::start_break(storage, time)?,
                false => commands::control::stop_break(storage, time)?,
            };
        }
        Opt::Cancel(opt) => {
            debug!("Cancel");
            commands::control::cancel(opt.path.unwrap_or(default_path))?;
        }
        Opt::Stats { storage, month } => {
            debug!("Stats of `{:?}`", storage);
            commands::stats::stats(&storage.unwrap_or(default_path), month)?;
        }
        Opt::Migrate(opt) => {
            debug!("Migrate");
            commands::control::migrate(opt.path.unwrap_or(default_path))?;
        }
        Opt::Configure(opt) => {
            let storage = opt.path.unwrap_or(default_path);
            debug!("Configure, stored in {:?}", storage);
            commands::config::configure(storage)?;
        }
    }

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    if let Err(e) = run() {
        if let Some(inner) = e.downcast_ref::<UsageError>() {
            log::error!("{}", inner);
            std::process::exit(1);
        } else {
            Err(e)
        }
    } else {
        Ok(())
    }
}
