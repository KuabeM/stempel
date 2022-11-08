use crate::errors::*;
use chrono::{DateTime, Utc};

pub fn parse_time(src: &str) -> Result<DateTime<Utc>> {
    let sign_pos = src.ends_with('+');
    let stripped = src
        .strip_suffix(|p| p == '+' || p == '-')
        .ok_or_else(|| eyre!("Does not end with + or -"))?;
    let human = stripped.parse::<humantime::Duration>()?;
    let duration = chrono::Duration::from_std(*human)?;

    let date_time: DateTime<Utc> = if sign_pos {
        Utc::now().checked_add_signed(duration)
    } else {
        Utc::now().checked_sub_signed(duration)
    }
    .ok_or_else(|| eyre!("Could not convert {} to duration", duration))?;
    log::trace!(
        "Deserialized {} to an offset {}min, timestamp {}",
        src,
        duration.num_minutes(),
        date_time
    );
    Ok(date_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn deserialize_min_add() {
        let input = "10m+";
        let time = parse_time(input); //.expect("Can parse");
        println!("{:?}", time);
        let expected = time.unwrap().signed_duration_since(Utc::now());
        assert!(expected < Duration::minutes(10));
        assert!(expected > Duration::seconds(60 * 9 + 59));
    }

    #[test]
    fn deserialize_min_sub() {
        let input = "10m-";
        let time = dbg!(parse_time(input).expect("Can parse"));
        let expected = time.signed_duration_since(Utc::now());
        assert!(expected < Duration::seconds(-9 * 60 - 59));
        assert!(expected > Duration::seconds(-10 * 60 - 1));
    }

    #[test]
    fn deserialize_full_fmt() {
        let input = "10h3m2s+";
        let time = dbg!(parse_time(input).expect("Can parse"));
        let expected = dbg!(time.signed_duration_since(Utc::now()));
        assert!(expected < dbg!(Duration::seconds(10 * 60 * 60 + 3 * 60 + 2)));
        assert!(expected > dbg!(Duration::seconds(10 * 60 * 60 + 3 * 60 + 1)));
    }

    #[test]
    fn deserialize_hoursseconds() {
        let input = "2h37s+";
        let time = dbg!(parse_time(input).expect("Can parse"));
        let expected = dbg!(time.signed_duration_since(Utc::now()));
        assert!(expected < dbg!(Duration::seconds(2 * 60 * 60 + 37)));
        assert!(expected > dbg!(Duration::seconds(2 * 60 * 60 + 35)));
    }

    #[test]
    fn deserialize_minutesseconds() {
        let input = "2m80s+";
        let time = dbg!(parse_time(input).expect("Can parse"));
        let expected = dbg!(time.signed_duration_since(Utc::now()));
        assert!(expected < dbg!(Duration::seconds(3 * 60 + 20)));
        assert!(expected > dbg!(Duration::seconds(3 * 60 + 18)));
    }

    #[test]
    fn deserialize_bad_format() {
        assert!(parse_time("10mS-").is_err());
        assert!(parse_time("10k+").is_err());
        assert!(parse_time("10m").is_err());
        assert!(parse_time("1-").is_err());
    }
}
