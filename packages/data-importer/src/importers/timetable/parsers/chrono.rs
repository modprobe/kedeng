use std::str::FromStr;

use chrono::{NaiveDate, NaiveTime};
use nom::{bytes::complete::take_while_m_n, combinator::map_res, AsChar, IResult, Parser};

pub fn date_string(input: &str) -> IResult<&str, NaiveDate> {
    let (input, day) = take_while_m_n(2, 2, AsChar::is_dec_digit)(input)?;
    let (input, month) = take_while_m_n(2, 2, AsChar::is_dec_digit)(input)?;
    let (input, year) = take_while_m_n(4, 4, AsChar::is_dec_digit)(input)?;

    Ok((
        input,
        NaiveDate::from_ymd_opt(
            i32::from_str(year).unwrap(),
            u32::from_str(month).unwrap(),
            u32::from_str(day).unwrap(),
        )
        .unwrap(),
    ))
}

pub fn time_string(input: &str) -> IResult<&str, NaiveTime> {
    let (input, hour) =
        map_res(take_while_m_n(2, 2, AsChar::is_dec_digit), u32::from_str).parse(input)?;

    let (input, minute) =
        map_res(take_while_m_n(2, 2, AsChar::is_dec_digit), u32::from_str).parse(input)?;

    Ok((
        input,
        NaiveTime::from_hms_opt(hour % 24, minute, 0).unwrap(),
    ))
}
