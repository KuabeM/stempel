//! Tune configuration via command line.
//!
//! Handler for the `config` subcommand.

use crate::errors::*;
use std::path::Path;

use crate::balance::{Config, TimeBalance};

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Number of months in stats: {}", self.month_stats)?;
        if let Some(d) = self.daily_hours {
            write!(f, "\nDaily working hours: {}", d)?;
        }
        Ok(())
    }
}

pub fn configure<P: AsRef<Path>>(storage: P) -> Result<()> {
    let mut balance = TimeBalance::from_file(&storage, true)?;
    let cfg = if let Some(cfg) = balance.config {
        println!("Current configuration:");
        println!("{}", cfg);
        cfg
    } else {
        println!("Nothing configured yet.");
        Config::default()
    };

    println!();
    println!("Let's change the configuration. Enter your desired value, leave blank for keeping the current value.");

    let mut input = String::new();
    println!("    Number of months to display ({}): ", cfg.month_stats);
    std::io::stdin()
        .read_line(&mut input)
        .wrap_err("Failed to read line from stdin")?;
    let month_history = input.trim().parse::<u8>().unwrap_or(cfg.month_stats);

    let daily_hours = cfg.daily_hours.unwrap_or_default();
    println!("    Daily working hours ({}): ", daily_hours);
    input.clear();
    std::io::stdin()
        .read_line(&mut input)
        .wrap_err("Failed to read line from stdin")?;
    let daily_hours = input.trim().parse::<u8>().unwrap_or(daily_hours);

    let weekly_stats = cfg.weekly_stats.unwrap_or_default();
    println!("    Print daily stats [y/n]: ({})", weekly_stats);
    input.clear();
    std::io::stdin()
        .read_line(&mut input)
        .wrap_err("Failed to read line from stdin")?;
    let weekly_stats = input.trim().contains('y');

    let cfg = Config {
        month_stats: month_history,
        daily_hours: Some(daily_hours),
        weekly_stats: Some(weekly_stats),
        //..cfg
    };
    log::trace!("Months to display {}", cfg.month_stats);
    log::trace!("Daily working hours {:?}", cfg.daily_hours);

    balance.config = Some(cfg);

    balance.canocicalize()?;
    balance.to_file(storage)?;

    Ok(())
}
