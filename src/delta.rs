use crate::errors::*;
use chrono::{DateTime, Utc};

pub fn parse_offset(src: &str) -> Result<DateTime<Utc>> {
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

pub fn parse_time(src: &str) -> Result<DateTime<Utc>> {
    let time = chrono::NaiveTime::parse_from_str(src, "%H:%M")?;
    let date_time = chrono::Utc::now().date_naive().and_time(time);
    let local = date_time.and_local_timezone(chrono::Local).unwrap();
    let utc = DateTime::<Utc>::from(local);
    log::trace!("Deserialized {} to a time point {}", src, date_time);
    Ok(utc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn deserialize_min_add() {
        let input = "10m+";
        let time = parse_offset(input); //.expect("Can parse");
        println!("{:?}", time);
        let expected = time.unwrap().signed_duration_since(Utc::now());
        assert!(expected < Duration::minutes(10));
        assert!(expected > Duration::seconds(60 * 9 + 59));
    }

    #[test]
    fn deserialize_min_sub() {
        let input = "10m-";
        let time = dbg!(parse_offset(input).expect("Can parse"));
        let expected = time.signed_duration_since(Utc::now());
        assert!(expected < Duration::seconds(-9 * 60 - 59));
        assert!(expected > Duration::seconds(-10 * 60 - 1));
    }

    #[test]
    fn deserialize_full_fmt() {
        let input = "10h3m2s+";
        let time = dbg!(parse_offset(input).expect("Can parse"));
        let expected = dbg!(time.signed_duration_since(Utc::now()));
        assert!(expected < dbg!(Duration::seconds(10 * 60 * 60 + 3 * 60 + 2)));
        assert!(expected > dbg!(Duration::seconds(10 * 60 * 60 + 3 * 60 + 1)));
    }

    #[test]
    fn deserialize_hoursseconds() {
        let input = "2h37s+";
        let time = dbg!(parse_offset(input).expect("Can parse"));
        let expected = dbg!(time.signed_duration_since(Utc::now()));
        assert!(expected < dbg!(Duration::seconds(2 * 60 * 60 + 37)));
        assert!(expected > dbg!(Duration::seconds(2 * 60 * 60 + 35)));
    }

    #[test]
    fn deserialize_minutesseconds() {
        let input = "2m80s+";
        let time = dbg!(parse_offset(input).expect("Can parse"));
        let expected = dbg!(time.signed_duration_since(Utc::now()));
        assert!(expected < dbg!(Duration::seconds(3 * 60 + 20)));
        assert!(expected > dbg!(Duration::seconds(3 * 60 + 18)));
    }

    #[test]
    fn deserialize_bad_format() {
        assert!(parse_offset("10mS-").is_err());
        assert!(parse_offset("10k+").is_err());
        assert!(parse_offset("10m").is_err());
        assert!(parse_offset("1-").is_err());
    }

    use chrono::Local;

    #[test]
    fn deserialize_time_works() {
        assert_eq!(
            parse_time("10:27").unwrap(),
            Utc::now()
                .date_naive()
                .and_hms_opt(10, 27, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        );
        assert_eq!(
            parse_time("13:00").unwrap(),
            Utc::now()
                .date_naive()
                .and_hms_opt(13, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        );
        assert_eq!(
            parse_time("1:4").unwrap(),
            Utc::now()
                .date_naive()
                .and_hms_opt(1, 4, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        );
        assert_eq!(
            parse_time("1:34").unwrap(),
            Utc::now()
                .date_naive()
                .and_hms_opt(1, 34, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        );
        assert_eq!(
            parse_time("00:59").unwrap(),
            Utc::now()
                .date_naive()
                .and_hms_opt(0, 59, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        );
        assert_eq!(
            parse_time("19:18").unwrap(),
            Utc::now()
                .date_naive()
                .and_hms_opt(19, 18, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        );
        assert_eq!(
            parse_time("1:00").unwrap(),
            Utc::now()
                .date_naive()
                .and_hms_opt(1, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
        );
    }

    #[test]
    fn deserialize_time_oob() {
        assert_eq!(
            parse_time("-10:27").unwrap_err().to_string(),
            "input contains invalid characters"
        );
        assert_eq!(
            parse_time("13:60").unwrap_err().to_string(),
            "input is out of range"
        );
        assert_eq!(
            parse_time("24:00").unwrap_err().to_string(),
            "input is out of range"
        );
        assert_eq!(
            parse_time("23:92").unwrap_err().to_string(),
            "input is out of range"
        );
        assert_eq!(
            parse_time("27:01").unwrap_err().to_string(),
            "input is out of range"
        );
    }
}
