//! Tune configuration via command line.
//!
//! Handler for the `config` subcommand.

use failure::Error;
use std::path::Path;

use crate::balance::{Config, TimeBalance};

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Number of months in stats: {}", self.month_stats)
    }
}

pub fn configure<P: AsRef<Path>>(storage: P) -> Result<(), Error> {
    let mut balance = TimeBalance::from_file(&storage, true)?;
    let cfg = if let Some(cfg) = balance.config {
        println!("Current configuration:");
        println!("    {}", cfg);
        cfg
    } else {
        println!("Nothing configured yet.");
        Config::default()
    };

    println!();
    println!("Let's change the configuration. Enter your desired value, leave blank for keeping the current value.");

    let mut input = String::new();
    println!("    Number of months to display ({}): ", cfg.month_stats);
    std::io::stdin().read_line(&mut input)?;
    let month_history = input.trim().parse::<u8>()?;
    let cfg = Config {
        month_stats: month_history,
        // ..cfg
    };

    balance.config = Some(cfg);

    balance.to_file(storage)?;

    Ok(())
}
