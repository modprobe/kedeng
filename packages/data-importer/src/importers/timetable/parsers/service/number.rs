use super::super::utils::Optional;
use nom::{
    AsChar, IResult, Parser,
    bytes::complete::{tag, take_while},
    character::complete::{char, line_ending},
    combinator::map_res,
    sequence::terminated,
};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct ServiceNumber {
    pub company_number: u32,
    pub service_number: u32,
    pub variant: Option<String>,
    pub first_stop: u32,
    pub last_stop: u32,
    pub name: Option<String>,
}

pub fn service_number(input: &str) -> IResult<&str, ServiceNumber> {
    let (input, _) = tag("%")(input)?;
    let (input, company_number) = map_res(
        terminated(take_while(AsChar::is_dec_digit), char(',')),
        u32::from_str,
    )
    .parse(input)?;

    let (input, service_number) = map_res(
        terminated(take_while(AsChar::is_dec_digit), char(',')),
        u32::from_str,
    )
    .parse(input)?;

    let (input, variant) =
        terminated(take_while(|c: char| c.is_ascii() && c != ','), char(',')).parse(input)?;

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

    let (input, name) =
        terminated(take_while(|c: char| c != '\r' && c != '\n'), line_ending).parse(input)?;

    Ok((
        input,
        ServiceNumber {
            company_number,
            service_number,
            variant: variant.trim().to_string().as_option(),
            first_stop,
            last_stop,
            name: name.trim().to_string().as_option(),
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_service_number() {
        const INPUT: &str = "%100,04084,      ,001,016,                              \r\n";
        let result = service_number(INPUT).expect("Failed to parse");

        assert_eq!(
            result,
            (
                "",
                ServiceNumber {
                    company_number: 100,
                    service_number: 4084,
                    variant: None,
                    first_stop: 1,
                    last_stop: 16,
                    name: None
                }
            )
        )
    }
}
