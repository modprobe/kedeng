use chrono::{Days, NaiveDate};
use nom::{
    AsChar, IResult, Parser,
    bytes::complete::{tag, take_while},
    character::complete::line_ending,
    combinator::map_res,
    multi::many0,
    sequence::terminated,
};
use std::str::FromStr;

use super::identification::{DeliveryIdentified, Identification, identification};

#[derive(Debug, PartialEq, Clone)]
pub struct Footnote {
    pub id: u32,
    pub vector: Vec<bool>,
}

impl<'a> Footnote {
    pub fn always_valid(delivery: &'a Identification) -> Footnote {
        Footnote {
            id: 0,
            vector: vec![true; delivery.days_valid() as usize],
        }
    }

    pub fn is_valid_on_date(&self, date: &NaiveDate, delivery: &Identification) -> bool {
        if *date < delivery.first_valid || *date > delivery.last_valid {
            return false;
        }

        let day_number = (*date - delivery.first_valid).num_days() as usize;
        *self.vector.get(day_number).unwrap_or(&false)
    }

    pub fn iterate_valid_dates(
        &'a self,
        delivery: &'a Identification,
    ) -> FootnoteValidDatesIter<'a> {
        FootnoteValidDatesIter {
            footnote: self,
            delivery,
            index: 0,
        }
    }
}

pub struct FootnoteValidDatesIter<'a> {
    footnote: &'a Footnote,
    delivery: &'a Identification,
    index: usize,
}

impl Iterator for FootnoteValidDatesIter<'_> {
    type Item = Option<NaiveDate>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.footnote.vector.len() {
            return None;
        }

        let is_valid_day = self.footnote.vector[self.index];
        if !is_valid_day {
            self.index += 1;
            return Some(None);
        }

        let result = Some(Some(
            self.delivery
                .first_valid
                .checked_add_days(Days::new(self.index as u64))
                .unwrap(),
        ));

        self.index += 1;

        result
    }
}

pub fn footnote(input: &str) -> IResult<&str, Footnote> {
    let (input, _) = tag("#")(input)?;

    let (input, id) = map_res(
        terminated(take_while(AsChar::is_dec_digit), line_ending),
        u32::from_str,
    )
    .parse(input)?;

    let (input, vector) = terminated(take_while(AsChar::is_bin_digit), line_ending).parse(input)?;
    let vector: Vec<bool> = vector.chars().map(|c| c == '1').collect();

    Ok((input, Footnote { id, vector }))
}

pub type Footnotes = DeliveryIdentified<Vec<Footnote>>;
impl Footnotes {
    pub fn get_by_id(&self, id: u32) -> Option<&Footnote> {
        self.data.iter().find(|f| f.id == id)
    }
}

pub fn footnote_file(input: &str) -> IResult<&str, Footnotes> {
    let (input, (identification, footnotes)) = (identification, many0(footnote)).parse(input)?;

    Ok((
        input,
        Footnotes {
            identification,
            data: footnotes,
        },
    ))
}

#[cfg(test)]
mod test {
    use crate::util::read_iso_8859_1_file;

    use super::*;

    #[test]
    fn it_parses_footnote() {
        const INPUT: &str = "#00000
11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111\r\n";

        let (rest_input, footnote) = footnote(INPUT).expect("failed to parse");

        assert_eq!(rest_input, "");
        assert_eq!(footnote.id, 0);
        assert_eq!(footnote.vector.len(), 251);
    }

    #[test]
    fn it_parses_footnotes_file() {
        let contents = read_iso_8859_1_file("./example/footnote.dat").unwrap();
        let (input, footnotes) = footnote_file(&contents).unwrap();

        assert_eq!(input, "");
        assert_eq!(footnotes.data.len(), 3709);

        for f in footnotes.data {
            assert_eq!(
                f.vector.len() as u64,
                footnotes.identification.days_valid(),
                "footnote {} represents a different number of days than the timetable should allow",
                f.id
            );
        }
    }

    #[test]
    fn it_iterates_over_valid_dates() {
        let footnote = Footnote {
            id: 1,
            vector: vec![true, true, false, false, false],
        };

        let delivery = Identification {
            company_number: "100".into(),
            first_valid: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            last_valid: NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(),
            version_number: "1".into(),
            description: "".into(),
        };

        let iter_result = footnote.iterate_valid_dates(&delivery).collect::<Vec<_>>();
        assert_eq!(
            iter_result,
            vec![
                Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
                Some(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()),
                None,
                None,
                None,
            ]
        );
    }
}
