use chrono::NaiveDate;
use nom::{
    AsChar, IResult, Parser,
    bytes::complete::{tag, take_while},
    character::complete::{char, line_ending},
    sequence::terminated,
};

use super::{chrono::date_string, utils::is_eol};

#[derive(Debug, PartialEq)]
pub struct DeliveryIdentified<T> {
    pub identification: Identification,
    pub data: T,
}

#[derive(Debug, PartialEq)]
pub struct Identification {
    pub company_number: String,
    pub first_valid: NaiveDate,
    pub last_valid: NaiveDate,
    pub version_number: String,
    pub description: String,
}

impl Identification {
    pub fn days_valid(&self) -> u64 {
        let days = (self.last_valid - self.first_valid).num_days() + 1;
        days.try_into().unwrap()
    }
}

pub fn identification(input: &str) -> IResult<&str, Identification> {
    let (input, _) = tag("@")(input)?;
    let (input, company_number) =
        terminated(take_while(AsChar::is_dec_digit), char(',')).parse(input)?;
    let (input, first_valid) = terminated(date_string, char(',')).parse(input)?;
    let (input, last_valid) = terminated(date_string, char(',')).parse(input)?;
    let (input, version_number) =
        terminated(take_while(AsChar::is_dec_digit), char(',')).parse(input)?;
    let (input, description) =
        terminated(take_while(|c: char| !is_eol(c)), line_ending).parse(input)?;

    Ok((
        input,
        Identification {
            company_number: company_number.to_string(),
            first_valid,
            last_valid,
            version_number: version_number.to_string(),
            description: description.to_string(),
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_identification() {
        const INPUT: &str = "@100,07042025,13122025,0070,IFF Standaard uit RIF\r\n";
        let result = identification(INPUT).expect("Failed to parse");

        assert_eq!(
            result,
            (
                "",
                Identification {
                    company_number: "100".to_string(),
                    first_valid: NaiveDate::from_ymd_opt(2025, 4, 7).unwrap(),
                    last_valid: NaiveDate::from_ymd_opt(2025, 12, 13).unwrap(),
                    version_number: "0070".to_string(),
                    description: "IFF Standaard uit RIF".to_string(),
                }
            )
        )
    }
}
