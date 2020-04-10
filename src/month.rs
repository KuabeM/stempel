use std::fmt;
use num_enum::{TryFromPrimitive, IntoPrimitive};
use std::str::FromStr;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Month {
    January = 1,
    Feburary,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<String> for Month {
    fn from(m: String) -> Self {
        Month::from_str(m.as_str()).expect("failed to convert string to month")
    }
}

impl FromStr for Month {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "january" => Ok(Month::January),
            "february" => Ok(Month::Feburary),
            "march" => Ok(Month::March),
            "april" => Ok(Month::April),
            "may" => Ok(Month::May),
            "june" => Ok(Month::June),
            "july" => Ok(Month::July),
            "august" => Ok(Month::August),
            "september" => Ok(Month::September),
            "october" => Ok(Month::October),
            "november" => Ok(Month::November),
            "december" => Ok(Month::December),
            &_ => Err(format!("Failed to parse {} into month", s)),
        }
    }
}

#[test]
fn display() {
    let jan = Month::January;
    assert_eq!(format!("{}", jan), "January");
}

#[test]
fn from_is_ok() {
    let string = "January".to_string();
    assert_eq!(Month::from(string), Month::January);
    let string = "december".to_string();
    assert_eq!(Month::from(string), Month::December);
    let string = "AprIL".to_string();
    assert_eq!(Month::from(string), Month::April);
}

#[test]
#[should_panic(expected = "failed to convert string to month")]
fn from_panics() {
    let bad_month = "some".to_string();
    Month::from(bad_month);
}

#[test]
fn try_from_primitive() {
    use std::convert::TryFrom;
    assert_eq!(Month::try_from(1), Ok(Month::January));
    assert_eq!(Month::try_from(12), Ok(Month::December));
    assert_eq!(Month::try_from(6), Ok(Month::June));
    assert!(Month::try_from(0).is_err());
    assert!(Month::try_from(13).is_err());
}

#[test]
fn from_str_works() {
    let m = "mAy";
    assert_eq!(Month::from_str(m), Ok(Month::May));
    let m = "something";
    assert!(Month::from_str(m).is_err());
}
