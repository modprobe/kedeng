use nom::{
    AsChar, IResult, Parser,
    bytes::complete::{tag, take_while},
    character::complete::{char, line_ending},
    combinator::map_res,
    sequence::terminated,
};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct Validity {
    pub footnote: u32,
    pub first_stop: u32,
    pub last_stop: u32,
}

pub fn validity(input: &str) -> IResult<&str, Validity> {
    let (input, _) = tag("-")(input)?;
    let (input, footnote) = map_res(
        terminated(take_while(AsChar::is_dec_digit), char(',')),
        u32::from_str,
    )
    .parse(input)?;

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
        Validity {
            footnote,
            first_stop,
            last_stop,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_validity_record() {
        const INPUT: &str = "-00001,000,999\r\n";
        let result = validity(INPUT).expect("failed to parse");

        assert_eq!(
            result,
            (
                "",
                Validity {
                    footnote: 1,
                    first_stop: 0,
                    last_stop: 999,
                }
            )
        )
    }
}
