use nom::{
    AsChar, IResult, Parser,
    bytes::complete::{tag, take_till, take_while},
    character::complete::{char, line_ending},
    combinator::map_res,
    sequence::terminated,
};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct Attribute {
    pub code: String,
    pub first_stop: u32,
    pub last_stop: u32,
    pub footnote: u32,
}

pub fn attribute(input: &str) -> IResult<&str, Attribute> {
    let (input, _) = tag("*")(input)?;
    let (input, code) = terminated(take_till(|c: char| c == ','), char(',')).parse(input)?;

    let (input, first_stop) = map_res(
        terminated(take_while(AsChar::is_dec_digit), char(',')),
        u32::from_str,
    )
    .parse(input)?;

    let (input, last_stop) = map_res(
        terminated(take_while(AsChar::is_dec_digit), char(',')),
        u32::from_str,
    )
    .parse(input)?;

    let (input, footnote) = map_res(
        terminated(take_while(AsChar::is_dec_digit), line_ending),
        u32::from_str,
    )
    .parse(input)?;

    Ok((
        input,
        Attribute {
            code: code.trim().to_string(),
            first_stop,
            last_stop,
            footnote,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_attribute() {
        const INPUT: &str = "*ROL ,001,007,00000\r\n";
        let result = attribute(INPUT).expect("failed to parse");

        assert_eq!(
            result,
            (
                "",
                Attribute {
                    code: "ROL".to_string(),
                    first_stop: 1,
                    last_stop: 7,
                    footnote: 0,
                }
            )
        )
    }
}
