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
        Commands::Break(startstop) => match startstop {
            clap_cli::StartStop::Start(t) => {
                commands::control::start_break(storage, t.time(), true)?
            }
            clap_cli::StartStop::Stop(t) => commands::control::stop_break(storage, t.time(), true)?,
            clap_cli::StartStop::Duration { dur } => commands::control::take_break(storage, dur)?,
        },
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
        Commands::Completions { shell } => {
            debug!("Generating shell completions for {}", shell);
            let mut app = <Cli as clap::CommandFactory>::command();
            let mut sink = std::io::stdout();
            let name = app.get_name().to_string();
            clap_complete::generate(shell, &mut app, name, &mut sink);
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
