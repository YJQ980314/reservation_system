use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationConflict {
    pub new: ResrvationWindow,
    pub old: ResrvationWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResrvationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(conflict) = s.parse() {
            Ok(Self::Parsed(conflict))
        } else {
            Ok(Self::Unparsed(s.to_string()))
        }
    }
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedInfo::from_str(s)?.try_into()
    }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();

    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: value.new.try_into()?,
            old: value.old.try_into()?,
        })
    }
}

impl TryFrom<HashMap<String, String>> for ResrvationWindow {
    type Error = ();

    // "Key (resource_id, timespan)=(ocean-view-room-713, [\"2023-12-26 22:00:00+00\",\"2023-12-30 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2023-12-25 22:00:00+00\",\"2023-12-28 19:00:00+00\"))."
    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let timespan_str = value.get("timespan").ok_or(())?.replace('"', "");
        let mut split = timespan_str.splitn(2, ',');
        let start = parse_datetime(split.next().ok_or(())?)?;
        let end = parse_datetime(split.next().ok_or(())?)?;
        Ok(Self {
            rid: value.get("resource_id").ok_or(())?.to_string(),
            start,
            end,
        })
    }
}

struct ParsedInfo {
    new: HashMap<String, String>,
    old: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use regular expression to parse the string
        let re = Regex::new(r#"\((?P<k1>[a-zA-Z0-9_-]+),\s*(?P<k2>[a-zA-Z0-9_-]+)\)=\((?P<v1>[a-zA-Z0-9_-]+)\s*,\s*\[(?P<v2>[^\)]+)\)"#).unwrap();
        let mut maps = vec![];
        for cap in re.captures_iter(s) {
            let mut map = HashMap::new();
            map.insert(cap["k1"].to_string(), cap["v1"].to_string());
            map.insert(cap["k2"].to_string(), cap["v2"].to_string());
            maps.push(Some(map));
        }

        if maps.len() != 2 {
            return Err(());
        }

        Ok(Self {
            new: maps[0].take().unwrap(),
            old: maps[1].take().unwrap(),
        })
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| ())?
        .with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::{ParsedInfo, ReservationConflictInfo};
    use crate::error::ResrvationWindow;
    use std::collections::HashMap;

    const ERR_MES: &str = "Key (resource_id, timespan)=(ocean-view-room-713, [\"2023-12-26 22:00:00+00\",\"2023-12-30 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2023-12-25 22:00:00+00\",\"2023-12-28 19:00:00+00\")).";

    #[test]
    fn parsed_info_should_work() {
        let info: ParsedInfo = ERR_MES.parse().unwrap();
        assert_eq!(info.new.get("resource_id").unwrap(), "ocean-view-room-713");
        assert_eq!(
            info.new.get("timespan").unwrap(),
            "\"2023-12-26 22:00:00+00\",\"2023-12-30 19:00:00+00\""
        );
        assert_eq!(info.old.get("resource_id").unwrap(), "ocean-view-room-713");
        assert_eq!(
            info.old.get("timespan").unwrap(),
            "\"2023-12-25 22:00:00+00\",\"2023-12-28 19:00:00+00\""
        );
    }

    #[test]
    fn hash_map_to_reserve_window_should_work() {
        let mut map = HashMap::new();
        map.insert("resource_id".to_string(), "ocean-view-room-713".to_string());
        map.insert(
            "timespan".to_string(),
            "\"2023-12-26 22:00:00+00\",\"2023-12-30 19:00:00+00\"".to_string(),
        );
        let window: ResrvationWindow = map.try_into().unwrap();
        assert_eq!(window.rid, "ocean-view-room-713");
        assert_eq!(window.start.to_string(), "2023-12-26 22:00:00 UTC");
        assert_eq!(window.end.to_string(), "2023-12-30 19:00:00 UTC");
    }

    #[test]
    fn conflict_error_message_should_parse() {
        let s = ERR_MES;
        let info: ReservationConflictInfo = s.parse().unwrap();
        match info {
            ReservationConflictInfo::Parsed(info) => {
                assert_eq!(info.new.rid, "ocean-view-room-713");
                assert_eq!(info.new.start.to_string(), "2023-12-26 22:00:00 UTC");
                assert_eq!(info.new.end.to_string(), "2023-12-30 19:00:00 UTC");
                assert_eq!(info.old.rid, "ocean-view-room-713");
                assert_eq!(info.old.start.to_string(), "2023-12-25 22:00:00 UTC");
                assert_eq!(info.old.end.to_string(), "2023-12-28 19:00:00 UTC");
            }
            ReservationConflictInfo::Unparsed(_) => {
                panic!("should be parsed");
            }
        }
    }
}
