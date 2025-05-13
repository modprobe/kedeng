use chrono::NaiveTime;
use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_till, take_until},
    character::complete::{char, line_ending, one_of},
    combinator::map,
    sequence::terminated,
};
use std::fmt::Display;

use crate::importers::timetable::parsers::{chrono::time_string, utils::is_eol};

#[derive(Debug, PartialEq, Clone)]
pub enum StationEventType {
    Departure,
    ShortStop,
    LongerStop,
    Passage,
    Arrival,
}

impl From<std::primitive::char> for StationEventType {
    fn from(value: char) -> Self {
        match value {
            '>' => Self::Departure,
            '.' => Self::ShortStop,
            ';' => Self::Passage,
            '+' => Self::LongerStop,
            '<' => Self::Arrival,
            _ => panic!("unknown station event type"),
        }
    }
}

impl Display for StationEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StationEventType::Departure => "DEPARTURE",
            StationEventType::ShortStop => "SHORT_STOP",
            StationEventType::LongerStop => "LONGER_STOP",
            StationEventType::Passage => "PASSAGE",
            StationEventType::Arrival => "ARRIVAL",
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StationEvent {
    pub stop_type: StationEventType,
    pub station: String,

    pub arrival_time: Option<NaiveTime>,
    pub departure_time: Option<NaiveTime>,
}

impl StationEvent {
    pub fn into_arrival(self) -> StationEvent {
        assert!(self.arrival_time.is_some());

        StationEvent {
            stop_type: StationEventType::Arrival,
            station: self.station.clone(),
            arrival_time: self.arrival_time.clone(),
            departure_time: None,
        }
    }

    pub fn into_departure(self) -> StationEvent {
        assert!(self.departure_time.is_some());

        StationEvent {
            stop_type: StationEventType::Departure,
            station: self.station.clone(),
            arrival_time: None,
            departure_time: self.departure_time.clone(),
        }
    }
}

pub fn station_event(input: &str) -> IResult<&str, StationEvent> {
    fn departure(input: &str) -> IResult<&str, StationEvent> {
        let (input, _) = tag(">")(input)?;
        let (input, station) = terminated(take_until(","), char(',')).parse(input)?;
        let (input, departure_time) = terminated(time_string, line_ending).parse(input)?;

        Ok((
            input,
            StationEvent {
                stop_type: StationEventType::Departure,
                station: station.trim().to_string(),
                arrival_time: None,
                departure_time: Some(departure_time),
            },
        ))
    }

    fn passage(input: &str) -> IResult<&str, StationEvent> {
        let (input, _) = tag(";")(input)?;
        let (input, station) = terminated(take_till(is_eol), line_ending).parse(input)?;

        Ok((
            input,
            StationEvent {
                stop_type: StationEventType::Passage,
                station: station.trim().to_string(),
                arrival_time: None,
                departure_time: None,
            },
        ))
    }

    fn short_stop(input: &str) -> IResult<&str, StationEvent> {
        let (input, _) = tag(".")(input)?;
        let (input, station) = terminated(take_until(","), char(',')).parse(input)?;
        let (input, time) = terminated(time_string, line_ending).parse(input)?;

        Ok((
            input,
            StationEvent {
                stop_type: StationEventType::ShortStop,
                station: station.trim().to_string(),
                arrival_time: Some(time),
                departure_time: Some(time),
            },
        ))
    }

    fn longer_stop(input: &str) -> IResult<&str, StationEvent> {
        let (input, _) = tag("+")(input)?;
        let (input, station) = terminated(take_until(","), char(',')).parse(input)?;

        let (input, arrival_time) = terminated(time_string, char(',')).parse(input)?;
        let (input, departure_time) = terminated(time_string, line_ending).parse(input)?;

        Ok((
            input,
            StationEvent {
                stop_type: StationEventType::LongerStop,
                station: station.trim().to_string(),
                arrival_time: Some(arrival_time),
                departure_time: Some(departure_time),
            },
        ))
    }

    fn arrival(input: &str) -> IResult<&str, StationEvent> {
        let (input, _) = tag("<")(input)?;
        let (input, station) = terminated(take_until(","), char(',')).parse(input)?;
        let (input, arrival_time) = terminated(time_string, line_ending).parse(input)?;

        Ok((
            input,
            StationEvent {
                stop_type: StationEventType::Arrival,
                station: station.trim().to_string(),
                arrival_time: Some(arrival_time),
                departure_time: None,
            },
        ))
    }

    let (_, stop_type) = map(one_of(">.;+<"), StationEventType::from).parse(input)?;

    match stop_type {
        StationEventType::Departure => departure(input),
        StationEventType::Passage => passage(input),
        StationEventType::ShortStop => short_stop(input),
        StationEventType::LongerStop => longer_stop(input),
        StationEventType::Arrival => arrival(input),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_station_event_departure() {
        const INPUT: &str = ">alm    ,1931\r\n";
        let result = station_event(INPUT).expect("failed to parse");

        assert_eq!(
            result,
            (
                "",
                StationEvent {
                    stop_type: StationEventType::Departure,
                    station: "alm".to_string(),
                    arrival_time: None,
                    departure_time: Some(NaiveTime::from_hms_opt(19, 31, 0).unwrap()),
                }
            )
        )
    }

    #[test]
    fn it_parses_station_event_passage() {
        const INPUT: &str = ";almm\r\n";
        let result = station_event(INPUT).expect("failed to parse");

        assert_eq!(
            result,
            (
                "",
                StationEvent {
                    stop_type: StationEventType::Passage,
                    station: "almm".to_string(),
                    arrival_time: None,
                    departure_time: None,
                }
            )
        )
    }

    #[test]
    fn it_parses_station_event_short_stop() {
        const INPUT: &str = ".ass    ,1959\r\n";
        let result = station_event(INPUT).expect("failed to parse");

        assert_eq!(
            result,
            (
                "",
                StationEvent {
                    stop_type: StationEventType::ShortStop,
                    station: "ass".to_string(),
                    arrival_time: Some(NaiveTime::from_hms_opt(19, 59, 0).unwrap()),
                    departure_time: Some(NaiveTime::from_hms_opt(19, 59, 0).unwrap()),
                }
            )
        )
    }

    #[test]
    fn it_parses_station_event_longer_stop() {
        const INPUT: &str = "+asd    ,1951,1953\r\n";
        let result = station_event(INPUT).expect("failed to parse");

        assert_eq!(
            result,
            (
                "",
                StationEvent {
                    stop_type: StationEventType::LongerStop,
                    station: "asd".to_string(),
                    arrival_time: Some(NaiveTime::from_hms_opt(19, 51, 0).unwrap()),
                    departure_time: Some(NaiveTime::from_hms_opt(19, 53, 0).unwrap()),
                }
            )
        )
    }

    #[test]
    fn it_parses_station_event_arrival() {
        const INPUT: &str = "<ekz    ,2053\r\n";
        let result = station_event(INPUT).expect("failed to parse");

        assert_eq!(
            result,
            (
                "",
                StationEvent {
                    stop_type: StationEventType::Arrival,
                    station: "ekz".to_string(),
                    arrival_time: Some(NaiveTime::from_hms_opt(20, 53, 0).unwrap()),
                    departure_time: None,
                }
            )
        )
    }
}
