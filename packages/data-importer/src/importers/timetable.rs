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
use atty::Stream::Stdout;
use chrono::{NaiveDateTime, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use nom::{IResult, Parser};
use postgres::Client;
use ratelimit::Ratelimiter;
use sea_query::{Expr, OnConflict, PostgresQueryBuilder, Query, ReturningClause};
use sea_query_postgres::PostgresBinder;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use uuid::Uuid;

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

    let timetable = load_file(&data_dir.join("./timetbls.dat"), timetable_file)?;
    println!("+ Loaded {} services", timetable.data.len());

    let footnotes = load_file(&data_dir.join("./footnote.dat"), footnote_file)?;
    println!("+ Loaded {} footnotes", footnotes.data.len());

    let companies = load_file(&data_dir.join("./company.dat"), company_file)?;
    println!("+ Loaded {} companies", companies.data.len());

    let services = timetable.data.iter().flat_map(Service::split_legs);

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner} [{elapsed_precise}] {msg}")?.tick_strings(&[
            "[    ]", "[=   ]", "[==  ]", "[=== ]", "[====]", "[ ===]", "[  ==]", "[   =]",
            "[    ]", "[   =]", "[  ==]", "[ ===]", "[====]", "[=== ]", "[==  ]", "[=   ]",
            "[====]",
        ]),
    );

    let output = |text: String| {
        if atty::is(Stdout) {
            spinner.println(text);
        } else {
            println!("{}", text);
        }
    };

    let ratelimit = Ratelimiter::builder(5, Duration::from_secs(1))
        .max_tokens(5)
        .build()?;

    for (sidx, service) in services.enumerate() {
        let mut transaction = db.transaction()?;

        let footnote = if service.validity.footnote == 0 {
            &Footnote::always_valid(&timetable.identification)
        } else {
            footnotes
                .get_by_id(service.validity.footnote)
                .context("! footnote not found")?
        };

        let service_number = (service.service_number.service_number != 0)
            .then_some(service.service_number.service_number.to_string())
            .or(service.service_number.variant.clone());

        if service_number.is_none() {
            output("+ Skipping service without number or variant".to_owned());
            continue;
        }

        let service_number = service_number.unwrap();

        spinner.set_message(format!(
            "[{} / ~{}] Processing service {} {}",
            sidx,
            timetable.data.len(),
            service.transport_mode.code,
            service_number
        ));

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
                service_number.clone().into(),
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

        let inserted_service = transaction
            .query(service_sql.as_str(), &service_params.as_params())
            .context("! failed to insert service(s)")?;

        assert_eq!(inserted_service.len(), 1);

        let service_id: Uuid = inserted_service.first().unwrap().get("id");

        let (journey_attributes, stop_attributes): (Vec<_>, Vec<_>) = service
            .attributes
            .iter()
            .cloned()
            .partition(|attr| attr.first_stop == 1 && attr.last_stop == service.num_stops());

        let mut journey_event_insert = Query::insert();
        journey_event_insert
            .into_table(db::JourneyEvent::Table)
            .columns([
                db::JourneyEvent::JourneyId,
                db::JourneyEvent::Station,
                db::JourneyEvent::EventTypePlanned,
                db::JourneyEvent::StopOrder,
                db::JourneyEvent::ArrivalTimePlanned,
                db::JourneyEvent::ArrivalPlatformPlanned,
                db::JourneyEvent::DepartureTimePlanned,
                db::JourneyEvent::DeparturePlatformPlanned,
                db::JourneyEvent::Attributes,
            ])
            .on_conflict(
                OnConflict::columns([db::JourneyEvent::JourneyId, db::JourneyEvent::StopOrder])
                    .update_columns([
                        db::JourneyEvent::Station,
                        db::JourneyEvent::EventTypePlanned,
                        db::JourneyEvent::ArrivalTimePlanned,
                        db::JourneyEvent::ArrivalPlatformPlanned,
                        db::JourneyEvent::DepartureTimePlanned,
                        db::JourneyEvent::DeparturePlatformPlanned,
                        db::JourneyEvent::Attributes,
                    ])
                    .to_owned(),
            );

        let mut at_least_one_journey_event = false;

        for journey in footnote
            .iterate_valid_dates(&timetable.identification)
            .flatten()
        {
            let mut journey_insert = Query::insert();
            let (journey_insert_sql, journey_insert_params) = journey_insert
                .into_table(db::Journey::Table)
                .columns([
                    db::Journey::ServiceId,
                    db::Journey::RunningOn,
                    db::Journey::Attributes,
                    db::Journey::SourceIds,
                ])
                .values_panic([
                    service_id.into(),
                    journey.into(),
                    (!journey_attributes.is_empty())
                        .then(|| {
                            journey_attributes
                                .iter()
                                .map(|attr| attr.code.clone())
                                .collect::<Vec<_>>()
                        })
                        .into(),
                    vec![service.service_identification.0.to_string()].into(),
                ])
                .on_conflict(
                    OnConflict::columns([db::Journey::ServiceId, db::Journey::RunningOn])
                        .update_columns([db::Journey::Attributes])
                        .value(
                            db::Journey::SourceIds,
                            Expr::cust("ARRAY(SELECT DISTINCT unnest(array_cat(\"journey\".\"source_ids\", \"excluded\".\"source_ids\")))"),
                        )
                        .to_owned(),
                )
                .returning(Query::returning().columns([db::Journey::Id]))
                .build_postgres(PostgresQueryBuilder);

            // output(journey_insert.to_string(PostgresQueryBuilder));

            let inserted_journey = transaction
                .query(
                    journey_insert_sql.as_str(),
                    &journey_insert_params.as_params(),
                )
                .context(format!(
                    "query: {} - params: {:?}",
                    journey_insert_sql.as_str(),
                    journey_insert_params.as_params()
                ))
                .context("! failed to insert journey")?;

            assert_eq!(inserted_journey.len(), 1);

            let journey_id: Uuid = inserted_journey.first().unwrap().get("id");

            for (idx, (event, platform)) in service.station_events.iter().enumerate() {
                at_least_one_journey_event = true;

                let stop_attributes = service.stop_number(event).and_then(|stop_number| {
                    let attribute_codes = stop_attributes
                        .iter()
                        .filter(|attr| {
                            attr.first_stop <= stop_number && stop_number <= attr.last_stop
                        })
                        .map(|attr| attr.code.clone())
                        .collect::<Vec<_>>();

                    (!attribute_codes.is_empty()).then_some(attribute_codes)
                });

                let (date, time) = (
                    journey,
                    event
                        .arrival_time
                        .or(event.departure_time)
                        .unwrap_or(Utc::now().naive_utc().time()),
                );

                journey_event_insert.values_panic([
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
                    stop_attributes.into(),
                ]);
            }
        }

        if at_least_one_journey_event {
            let journey_event_insert_query = journey_event_insert.to_string(PostgresQueryBuilder);
            transaction
                .batch_execute(journey_event_insert_query.as_str())
                .context(format!("query: {journey_event_insert_query}"))
                .context("! could not insert journey events")?;
        }

        while let Err(time_to_wait) = ratelimit.try_wait() {
            sleep(time_to_wait);
        }

        transaction
            .commit()
            .context("! could not commit transaction")?;

        output(format!(
            "+ Inserted service {} {} (from # {})",
            service.transport_mode.code,
            service_number.clone(),
            service.service_identification.0,
        ));
        spinner.tick();
    }

    println!("+ All done!");

    Ok(())
}
