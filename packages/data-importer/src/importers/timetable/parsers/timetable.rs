use nom::{
    IResult, Parser,
    combinator::opt,
    multi::{fold_many0, many0},
};

use super::{
    identification::{DeliveryIdentified, identification},
    service::{
        Service, attribute::attribute, identification::service_identification,
        number::service_number, platform_info::platform_info, station_event::station_event,
        transport_mode::transport_mode, validity::validity,
    },
};

pub type Timetable = DeliveryIdentified<Vec<Service>>;

pub fn timetable_file(input: &str) -> IResult<&str, Timetable> {
    let (input, (identification, services)) = (
        identification,
        fold_many0(
            (
                service_identification,
                many0(service_number),
                validity,
                transport_mode,
                many0(attribute),
                many0((station_event, opt(platform_info))),
            ),
            Vec::new,
            |mut acc: Vec<_>, item| {
                let (
                    identification,
                    service_number,
                    validity,
                    transport_mode,
                    attributes,
                    station_events,
                ) = item;
                acc.push(Service {
                    identification,
                    service_number,
                    validity,
                    transport_mode,
                    attributes,
                    station_events,
                });
                acc
            },
        ),
    )
        .parse(input)?;

    Ok((
        input,
        Timetable {
            identification,
            data: services,
        },
    ))
}

#[cfg(test)]
mod test {
    use crate::importers::timetable::parsers::service::identification::ServiceIdentification;

    use super::*;

    #[test]
    fn it_parses_minimal_file() {
        const INPUT: &str = "@100,07042025,13122025,0070,IFF Standaard uit RIF         \r
#00000001\r
%100,04084,      ,001,016,                              \r
-00001,000,999\r
&SPR ,001,016\r
>rtd    ,2324\r
?16   ,16   ,00001\r
.rtn    ,2329\r
?1    ,1    ,00001\r
+rta    ,2333,2334\r
?1    ,1    ,00001\r
.cps    ,2337\r
?1    ,1    ,00001\r
.nwk    ,2341\r
?1    ,1    ,00001\r
+gd     ,2348,2349\r
?3    ,3    ,00001\r
.gdg    ,2352\r
?1    ,1    ,00001\r
+wd     ,2401,2403\r
?6    ,6    ,00001\r
.bkl    ,2411\r
?2    ,2    ,00001\r
.ac     ,2419\r
?2    ,2    ,00001\r
.ashd   ,2422\r
?2    ,2    ,00001\r
+asb    ,2425,2427\r
?3    ,3    ,00001\r
.dvd    ,2430\r
?5    ,5    ,00001\r
.asa    ,2434\r
?1    ,1    ,00001\r
.asdm   ,2437\r
?8    ,8    ,00001\r
<asd    ,2442\r
?7b   ,7b   ,00001\r
";

        let (rest_input, timetable) = timetable_file(INPUT).expect("failed to parse");
        assert!(rest_input.is_empty());
        assert_eq!(timetable.data.len(), 1);

        let service = timetable.data.first().unwrap();
        assert_eq!(service.identification, ServiceIdentification(1));
        assert_eq!(service.service_number.len(), 1);
    }
}
