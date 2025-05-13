use nom::{
    AsChar, IResult, Parser,
    bytes::complete::{tag, take_while},
    character::complete::{char, line_ending},
    combinator::map_res,
    sequence::terminated,
};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct TransportMode {
    pub code: String,
    pub first_stop: u32,
    pub last_stop: u32,
}

pub fn transport_mode(input: &str) -> IResult<&str, TransportMode> {
    let (input, _) = tag("&")(input)?;

    let (input, code) =
        terminated(take_while(|c: char| c.is_ascii() && c != ','), char(',')).parse(input)?;

    let (input, first_stop) = map_res(
        terminated(take_while(AsChar::is_dec_digit), char(',')),
        u32::from_str,
    )
    .parse(input)?;

    let (input, last_stop) = map_res(
        terminated(take_while(AsChar::is_dec_digit), line_ending),
        u32::from_str,
    )
    .parse(input)?;

    Ok((
        input,
        TransportMode {
            code: code.trim().to_string(),
            first_stop,
            last_stop,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_transport_mode() {
        const INPUT: &str = "&SPR ,001,005\r\n";
        let result = transport_mode(INPUT).expect("failed to parse");

        assert_eq!(
            result,
            (
                "",
                TransportMode {
                    code: "SPR".to_string(),
                    first_stop: 1,
                    last_stop: 5
                }
            )
        )
    }
}
