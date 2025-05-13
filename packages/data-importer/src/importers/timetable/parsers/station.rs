use nom::{
    IResult, Parser,
    bytes::complete::{take_till, take_until},
    character::complete::{bin_digit1, char, line_ending},
    combinator::map_res,
    multi::{many_m_n, many0},
    sequence::terminated,
};
use std::str::FromStr;

use super::{
    identification::{DeliveryIdentified, identification},
    utils::is_eol,
};

#[derive(Debug, PartialEq)]
pub struct Station {
    pub code: String,
    pub name: String,
    pub country: String,
    pub is_interchange: bool,
    pub layover_minimum_minutes: u8,
}

pub fn station(input: &str) -> IResult<&str, Station> {
    let (input, is_interchange) = terminated(bin_digit1, char(',')).parse(input)?;
    let is_interchange = is_interchange == "1";

    let (input, code) = terminated(take_until(","), char(',')).parse(input)?;
    let code = code.trim().to_string();

    let (input, layover_minimum_minutes) =
        terminated(map_res(take_until(","), u8::from_str), char(',')).parse(input)?;

    let (input, _) = terminated(take_until(","), char(',')).parse(input)?;

    let (input, country) = terminated(take_until(","), char(',')).parse(input)?;
    let country = country.trim().to_string();

    let (input, _) = many_m_n(4, 4, terminated(take_until(","), char(','))).parse(input)?;

    let (input, name) = terminated(take_till(is_eol), line_ending).parse(input)?;
    let name = name.trim().to_string();

    Ok((
        input,
        Station {
            code,
            name,
            country,
            is_interchange,
            layover_minimum_minutes,
        },
    ))
}

pub type Stations = DeliveryIdentified<Vec<Station>>;

pub fn station_file(input: &str) -> IResult<&str, Stations> {
    let (input, (identification, stations)) = (identification, many0(station)).parse(input)?;
    Ok((
        input,
        Stations {
            identification,
            data: stations,
        },
    ))
}

#[cfg(test)]
mod test {
    use crate::util::read_iso_8859_1_file;

    use super::*;

    #[test]
    fn it_parses_station() {
        const INPUT: &str = "1,ac     ,02,02,NL  ,0000,  ,012701,047683,Abcoude\r\n";
        let (rest_input, station) = station(INPUT).expect("failed to parse");

        assert!(rest_input.is_empty());
        assert_eq!(
            station,
            Station {
                code: "ac".to_string(),
                name: "Abcoude".to_string(),
                country: "NL".to_string(),
                is_interchange: true,
                layover_minimum_minutes: 2,
            }
        )
    }

    #[test]
    fn it_parses_station_file() {
        let input = read_iso_8859_1_file("./example/stations.dat").unwrap();
        let (rest_input, stations) = station_file(&input).unwrap();

        assert!(rest_input.is_empty());
        assert_eq!(stations.data.len(), 666);
    }
}
