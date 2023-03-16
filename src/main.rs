use log::debug;
use std::path::PathBuf;

use stempel::commands;
use stempel::errors::UsageError;

mod clap_cli;
use clap_cli::*;

fn run() -> color_eyre::Result<()> {
    let clap = Cli::parse();

    let fallback = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()));
    let default_path = dirs::config_dir().unwrap_or(fallback).join("stempel.json");

    let storage = clap.storage.unwrap_or(default_path);
    match clap.command {
        Commands::Start(timings) => {
            let time_pt = timings.time();
            debug!("Start at {}, store in {:?}", time_pt, storage);
            commands::control::start(storage, time_pt)?;
        }
        Commands::Stop(timings) => {
            let time_pt = timings.time();
            debug!("Stop at {:?}, store in {:?}", time_pt, storage);
            commands::control::stop(storage, time_pt)?;
        }
        Commands::Break(startstop) => {
            let (is_start, time) = match startstop {
                clap_cli::StartStop::Start(t) => (true, t.time()),
                clap_cli::StartStop::Stop(t) => (false, t.time()),
            };
            debug!("Break at {}, store in {:?}", time, storage);
            match is_start {
                true => commands::control::start_break(storage, time)?,
                false => commands::control::stop_break(storage, time)?,
            };
        }
        Commands::Cancel => {
            debug!("Cancel");
            commands::control::cancel(storage)?;
        }
        Commands::Stats { month } => {
            debug!("Stats of `{:?}`", month);
            commands::stats::stats(storage, month)?;
        }
        Commands::Migrate => {
            debug!("Migrate, stored in {:?}", storage);
            commands::control::migrate(storage)?;
        }
        Commands::Configure => {
            debug!("Configure, stored in {:?}", storage);
            commands::config::configure(storage)?;
        }
    };

    Ok(())
}

fn main() -> color_eyre::Result<()> {
    env_logger::init();
    color_eyre::install()?;
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
