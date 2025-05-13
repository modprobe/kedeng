use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_till, take_until},
    character::complete::{char, line_ending},
    combinator::map_res,
    sequence::terminated,
};
use std::str::FromStr;

use crate::importers::timetable::parsers::utils::is_eol;

#[derive(Debug, PartialEq, Clone)]
pub struct PlatformInfo {
    pub arrival_platform: String,
    pub departure_platform: String,
    pub footnote: u32,
}

pub fn platform_info(input: &str) -> IResult<&str, PlatformInfo> {
    let (input, _) = tag("?")(input)?;

    let (input, arrival_platform) = terminated(take_until(","), char(',')).parse(input)?;
    let (input, departure_platform) = terminated(take_until(","), char(',')).parse(input)?;

    let (input, footnote) =
        map_res(terminated(take_till(is_eol), line_ending), u32::from_str).parse(input)?;

    Ok((
        input,
        PlatformInfo {
            arrival_platform: arrival_platform.trim().to_string(),
            departure_platform: departure_platform.trim().to_string(),
            footnote,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_platform_info() {
        const INPUT: &str = "?1a   ,1a   ,00082\r\n";
        let result = platform_info(INPUT).expect("failed to parse");

        assert_eq!(
            result,
            (
                "",
                PlatformInfo {
                    arrival_platform: "1a".to_string(),
                    departure_platform: "1a".to_string(),
                    footnote: 82,
                }
            )
        )
    }
}
