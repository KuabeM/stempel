/// Parsing and printing from and to cli.
use std::str::FromStr;

use crate::errors::*;

#[derive(Debug)]
pub enum YesNo {
    Yes,
    No,
}

impl FromStr for YesNo {
    type Err = color_eyre::eyre::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let lower = input.trim().to_lowercase();
        match lower.as_str() {
            "yes" | "y" => Ok(Self::Yes),
            "no" | "n" => Ok(Self::No),
            e => Err(eyre!("Failed to parse {} into 'yes' or 'no'", e)),
        }
    }
}

impl YesNo {
    pub fn wait_for_decision() -> Result<Self> {
        let yes = loop {
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .wrap_err("Failed to read line from stdin")?;
            if let Ok(yn) = crate::cli_input::YesNo::from_str(&input) {
                log::trace!("Parsed {:?}", yn);
                break yn;
            }
        };
        Ok(yes)
    }
}
