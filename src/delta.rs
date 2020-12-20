use chrono::{DateTime, Duration, Utc};
use failure::{format_err, Error};
use regex::{Captures, Regex};

#[derive(Debug, Eq, PartialEq)]
pub struct OffsetTime {
    pub date_time: DateTime<Utc>,
}

impl Default for OffsetTime {
    fn default() -> Self {
        Self {
            date_time: Utc::now(),
        }
    }
}

impl std::fmt::Display for OffsetTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.date_time)
    }
}

pub fn parse_time(src: &str) -> Result<OffsetTime, Error> {
    let regex: Regex = Regex::new(r"([0-9]+)([h|m|s])([\+-])").unwrap();

    let duration = regex
        .captures(src)
        .ok_or_else(|| Err::<Captures, Error>(format_err!("Failed to parse {} to DateTime", src)))
        .map(|captures| {
            if captures.len() == 4 {
                let number = &captures[1].parse::<u64>().unwrap();
                let unit = &captures[2];
                let sign = if &captures[3] == "+" { 1 } else { -1 };
                match unit {
                    "h" => Duration::hours(*number as i64 * sign),
                    "m" => Duration::minutes(*number as i64 * sign),
                    "s" => Duration::seconds(*number as i64 * sign),
                    _ => unreachable!("Not covered by regex"),
                }
            } else {
                Duration::seconds(0)
            }
        })
        .map_err(|e| format_err!("Regex: {:?}", e))?;

    let date_time: DateTime<Utc> = Utc::now()
        .checked_add_signed(duration)
        .ok_or_else(|| format_err!("Failed to construct DateTime"))?;
    Ok(OffsetTime { date_time })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_min_add() {
        let input = "10m+";
        let time = parse_time(input).expect("Can parse");
        let expected = time.date_time.signed_duration_since(Utc::now());
        assert!(expected < Duration::minutes(10));
        assert!(expected > Duration::seconds(60 * 9 + 59));
    }

    #[test]
    fn deserialize_min_sub() {
        let input = "10m-";
        let time = dbg!(parse_time(input).expect("Can parse"));
        let expected = time.date_time.signed_duration_since(Utc::now());
        assert!(expected < Duration::seconds(-9 * 60 - 59));
        assert!(expected > Duration::seconds(-10 * 60 - 1));
    }
}
