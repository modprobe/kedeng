pub mod parsers;

use crate::db;
use crate::importers::timetable::parsers::footnote::Footnote;
use crate::importers::timetable::parsers::service::Service;
use crate::importers::timetable::parsers::{
    company::company_file, footnote::footnote_file, identification::DeliveryIdentified,
    timetable::timetable_file,
};
use crate::util::read_iso_8859_1_file;
use anyhow::{Context, Result, anyhow};
use chrono::{NaiveDateTime, Utc};
use indicatif::ProgressBar;
use nom::IResult;
use postgres::Client;
use sea_query::{OnConflict, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::{env, fs};
use uuid::{ContextV7, Timestamp, Uuid};

fn load_file<TData>(
    path: &Path,
    parser: impl Fn(&str) -> IResult<&str, DeliveryIdentified<TData>>,
) -> Result<DeliveryIdentified<TData>> {
    let file_contents = read_iso_8859_1_file(path.to_str().unwrap())?;
    let (_, data) = parser(&file_contents).or(Err(anyhow!("! failed to parse file")))?;

    Ok(data)
}

const DATA_URL: &str = "https://data.ndovloket.nl/ns/ns-latest.zip";
fn download_and_extract_data(dir: &Path) -> Result<()> {
    let timetable_data_archive: Vec<u8> = ureq::get(DATA_URL)
        .call()
        .context("! failed to download timetable zip")?
        .body_mut()
        .read_to_vec()?;

    zip_extract::extract(Cursor::new(timetable_data_archive), dir, true)
        .context("! failed to extract zip")?;

    Ok(())
}

pub fn import(db: &mut Client, input_path: Option<String>) -> Result<()> {
    let data_dir: PathBuf = if let Some(input_path) = input_path {
        println!("+ Using input path: {}", input_path);
        PathBuf::from(input_path)
    } else {
        println!("+ Downloading and unzipping latest data");
        let data_dir = env::temp_dir().join("./kedeng-data-importer");
        fs::create_dir_all(&data_dir).context("! failed to create temp dir")?;
        println!("+ Created temp dir: {}", data_dir.display());

        download_and_extract_data(&data_dir)?;

        data_dir
    };

    let timetable = load_file(&data_dir.join("./timetbls_new.dat"), timetable_file)?;
    println!("+ Loaded {} services", timetable.data.len());

    let footnotes = load_file(&data_dir.join("./footnote.dat"), footnote_file)?;
    println!("+ Loaded {} footnotes", footnotes.data.len());

    let companies = load_file(&data_dir.join("./company.dat"), company_file)?;
    println!("+ Loaded {} companies", companies.data.len());

    let services = timetable.data.iter().flat_map(Service::split_legs);
    let bar = ProgressBar::new(services.size_hint().0 as u64);

    let uuid_ctx = ContextV7::new();

    for service in services {
        let footnote = if service.validity.footnote == 0 {
            &Footnote::always_valid(&timetable.identification)
        } else {
            footnotes
                .get_by_id(service.validity.footnote)
                .context("! footnote not found")?
        };

        if service.service_number.service_number == 0 && service.service_number.variant.is_none() {
            println!("+ Skipping service without number or variant");
            continue;
        }

        let company = companies
            .get_by_id(service.service_number.company_number)
            .unwrap();

        let (service_sql, service_params) = Query::insert()
            .into_table(db::Service::Table)
            .columns([
                db::Service::TrainNumber,
                db::Service::Type,
                db::Service::Provider,
            ])
            .values_panic([
                if service.service_number.service_number != 0 {
                    service.service_number.service_number.to_string().into()
                } else {
                    service.service_number.clone().variant.unwrap().into()
                },
                service.transport_mode.code.clone().into(),
                company.code.clone().into(),
            ])
            .on_conflict(
                OnConflict::columns([db::Service::TrainNumber, db::Service::TimetableYear])
                    // we actually don't need to update anything, but with `DO NOTHING` we
                    // will not get anything back from the RETURNING clause on a conflict
                    .update_column(db::Service::TrainNumber)
                    .to_owned(),
            )
            .returning(Query::returning().column(db::Service::Id))
            .build_postgres(PostgresQueryBuilder);

        let inserted_service = db
            .query(service_sql.as_str(), &service_params.as_params())
            .context("! failed to insert service(s)")?;
        assert_eq!(inserted_service.len(), 1);

        let service_id: Uuid = inserted_service.first().unwrap().get("id");

        let mut journey_insert = Query::insert();
        journey_insert.into_table(db::Journey::Table).columns([
            db::Journey::Id,
            db::Journey::ServiceId,
            db::Journey::RunningOn,
        ]);

        let mut journey_event_insert = Query::insert();
        journey_event_insert
            .into_table(db::JourneyEvent::Table)
            .columns([
                db::JourneyEvent::Id,
                db::JourneyEvent::JourneyId,
                db::JourneyEvent::Station,
                db::JourneyEvent::EventType,
                db::JourneyEvent::StopOrder,
                db::JourneyEvent::ArrivalTimePlanned,
                db::JourneyEvent::ArrivalPlatformPlanned,
                db::JourneyEvent::DepartureTimePlanned,
                db::JourneyEvent::DeparturePlatformPlanned,
            ]);

        let mut at_least_one_journey = false;
        let mut at_least_one_journey_event = false;

        for journey in footnote
            .iterate_valid_dates(&timetable.identification)
            .flatten()
        {
            at_least_one_journey = true;

            let journey_id = Uuid::new_v7(Timestamp::from_unix(
                &uuid_ctx,
                NaiveDateTime::from(journey).and_utc().timestamp() as u64,
                0,
            ));

            journey_insert.values_panic([
                journey_id.into(),
                service_id.into(),
                journey.format("%Y-%m-%d").to_string().into(),
            ]);

            for (idx, (event, platform)) in service.station_events.iter().enumerate() {
                at_least_one_journey_event = true;

                let (date, time) = (
                    journey,
                    event
                        .arrival_time
                        .or(event.departure_time)
                        .unwrap_or(Utc::now().naive_utc().time()),
                );

                let timestamp = Timestamp::from_unix(
                    &uuid_ctx,
                    date.and_time(time).and_utc().timestamp() as u64,
                    0,
                );

                journey_event_insert.values_panic([
                    Uuid::new_v7(timestamp).into(),
                    journey_id.into(),
                    event.station.clone().into(),
                    event.stop_type.to_string().into(),
                    (idx as u64).into(),
                    event.arrival_time.into(),
                    if platform.is_some() {
                        platform.clone().unwrap().arrival_platform.into()
                    } else {
                        None::<String>.into()
                    },
                    event.departure_time.into(),
                    if platform.is_some() {
                        platform.clone().unwrap().departure_platform.into()
                    } else {
                        None::<String>.into()
                    },
                ]);
            }
        }

        let mut transaction = db.transaction()?;

        if at_least_one_journey {
            let journey_insert_query = journey_insert.to_string(PostgresQueryBuilder);
            transaction
                .batch_execute(journey_insert_query.as_str())
                .context(format!("query: {journey_insert_query}"))
                .context("! could not insert journey")?;
        }

        if at_least_one_journey_event {
            let journey_event_insert_query = journey_event_insert.to_string(PostgresQueryBuilder);
            transaction
                .batch_execute(journey_event_insert_query.as_str())
                .context(format!("query: {journey_event_insert_query}"))
                .context("! could not insert journey events")?;
        }

        transaction
            .commit()
            .context("! could not commit transaction")?;

        bar.inc(1);
    }

    println!("+ All done!");

    Ok(())
}
