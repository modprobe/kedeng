use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_till},
    character::complete::line_ending,
    sequence::delimited,
};
use std::str::FromStr;

use crate::importers::timetable::parsers::utils::is_eol;

#[derive(Debug, PartialEq, Clone)]
pub struct ServiceIdentification(pub u32);

pub fn service_identification(input: &str) -> IResult<&str, ServiceIdentification> {
    let (input, id) = delimited(tag("#"), take_till(is_eol), line_ending).parse(input)?;

    Ok((input, ServiceIdentification(u32::from_str(id).unwrap())))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_service_identification() {
        const INPUT: &str = "#00000001\r\n";
        let result = service_identification(INPUT).expect("Failed to parse");

        assert_eq!(result, ("", ServiceIdentification(1)))
    }
}
