use nom::{
    IResult, Parser,
    bytes::complete::{take_till, take_until},
    character::complete::{char, line_ending},
    combinator::map_res,
    multi::many0,
    sequence::terminated,
};
use std::str::FromStr;

use super::{
    identification::{DeliveryIdentified, identification},
    utils::is_eol,
};

#[derive(Debug, PartialEq)]
pub struct Company {
    pub id: u32,
    pub code: String,
    pub name: String,
}

pub type Companies = DeliveryIdentified<Vec<Company>>;
impl Companies {
    pub fn get_by_id(&self, id: u32) -> Option<&Company> {
        self.data.iter().find(|c| c.id == id)
    }
}

pub fn company(input: &str) -> IResult<&str, Company> {
    let (input, (id, code, name)) = (
        map_res(terminated(take_until(","), char(',')), u32::from_str),
        terminated(take_until(","), char(',')),
        terminated(take_until(","), char(',')),
    )
        .parse(input)?;

    let (input, _) = terminated(take_till(is_eol), line_ending).parse(input)?;

    Ok((
        input,
        Company {
            id,
            code: code.trim().to_string(),
            name: name.trim().to_string(),
        },
    ))
}

pub fn company_file(input: &str) -> IResult<&str, Companies> {
    let (input, (identification, companies)) = (identification, many0(company)).parse(input)?;
    Ok((
        input,
        Companies {
            identification,
            data: companies,
        },
    ))
}

#[cfg(test)]
mod test {
    use crate::util::read_iso_8859_1_file;

    use super::*;

    #[test]
    fn it_parses_company() {
        const INPUT: &str = "970,CFL       ,Chemins de Fer Luxembourg     ,0000\r\n";
        let (rest_input, company) = company(INPUT).expect("failed to parse");

        assert!(rest_input.is_empty());
        assert_eq!(
            company,
            Company {
                id: 970,
                code: "CFL".into(),
                name: "Chemins de Fer Luxembourg".into(),
            }
        )
    }

    #[test]
    fn it_parses_company_file() {
        let input = read_iso_8859_1_file("./example/company.dat").unwrap();
        let (rest_input, companies) = company_file(&input).expect("failed to parse");

        assert!(rest_input.is_empty());
        assert_eq!(companies.data.len(), 67);
    }
}
