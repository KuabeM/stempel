use crate::errors::{Result, TimeErr};
use chrono::{DateTime, Duration, Utc};
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

pub fn parse_time(src: &str) -> Result<OffsetTime> {
    let regex: Regex = Regex::new(r"(([0-9]+)[h])?(([0-9]+)[m])?(([0-9]+)[s])?([\+|-])").unwrap();

    let duration = regex
        .captures(src)
        .ok_or_else(|| {
            Err::<Captures, TimeErr>(TimeErr::Parse(format!(
                "Failed to parse {} to DateTime",
                src
            )))
        })
        .map(|captures| {
            if captures.len() == 8 {
                let h = &captures
                    .get(2)
                    .map(|m| m.as_str())
                    .unwrap_or("0")
                    .parse::<i64>()
                    .unwrap();
                let m = &captures
                    .get(4)
                    .map(|m| m.as_str())
                    .unwrap_or("0")
                    .parse::<i64>()
                    .unwrap();
                let s = &captures
                    .get(6)
                    .map(|m| m.as_str())
                    .unwrap_or("0")
                    .parse::<i64>()
                    .unwrap();
                let sign = if &captures[7] == "+" { 1 } else { -1 };
                Duration::seconds((h * 3600 + m * 60 + s) * sign)
            } else {
                Duration::seconds(0)
            }
        })
        .map_err(|e| TimeErr::Parse(format!("Regex: {:?}", e)))?;
    if duration.num_seconds() == 0 {
        return Err(TimeErr::Parse(format!(
            "Failed to deserialize offset '{}'",
            src
        )));
    }

    let date_time: DateTime<Utc> = Utc::now()
        .checked_add_signed(duration)
        .ok_or_else(|| TimeErr::Parse("Failed to construct DateTime".into()))?;
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

    #[test]
    fn deserialize_full_fmt() {
        let input = "10h3m2s+";
        let time = dbg!(parse_time(input).expect("Can parse"));
        let expected = dbg!(time.date_time.signed_duration_since(Utc::now()));
        assert!(expected < dbg!(Duration::seconds(10 * 60 * 60 + 3 * 60 + 2)));
        assert!(expected > dbg!(Duration::seconds(10 * 60 * 60 + 3 * 60 + 1)));
    }

    #[test]
    fn deserialize_hoursseconds() {
        let input = "2h37s+";
        let time = dbg!(parse_time(input).expect("Can parse"));
        let expected = dbg!(time.date_time.signed_duration_since(Utc::now()));
        assert!(expected < dbg!(Duration::seconds(2 * 60 * 60 + 37)));
        assert!(expected > dbg!(Duration::seconds(2 * 60 * 60 + 35)));
    }

    #[test]
    fn deserialize_minutesseconds() {
        let input = "2m80s+";
        let time = dbg!(parse_time(input).expect("Can parse"));
        let expected = dbg!(time.date_time.signed_duration_since(Utc::now()));
        assert!(expected < dbg!(Duration::seconds(3 * 60 + 20)));
        assert!(expected > dbg!(Duration::seconds(3 * 60 + 18)));
    }

    #[test]
    fn deserialize_bad_format() {
        assert!(parse_time("10ms-").is_err());
        assert!(parse_time("10k+").is_err());
        assert!(parse_time("10m").is_err());
        assert!(parse_time("1-").is_err());
    }
}
