pub mod parsers;

use crate::db;
use crate::importers::timetable::parsers::company::Companies;
use crate::importers::timetable::parsers::footnote::{Footnote, Footnotes};
use crate::importers::timetable::parsers::service::{Service, ServiceLeg};
use crate::importers::timetable::parsers::timetable::Timetable;
use crate::importers::timetable::parsers::{
    company::company_file, footnote::footnote_file, identification::DeliveryIdentified,
    timetable::timetable_file,
};
use crate::util::read_iso_8859_1_file;
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use deadpool_postgres::Pool;
use nom::{IResult, Parser};
use sea_query::{Expr, OnConflict, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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
async fn download_and_extract_data(dir: &Path) -> Result<()> {
    let timetable_data_archive = reqwest::get(DATA_URL).await?.bytes().await?.to_vec();

    zip_extract::extract(Cursor::new(timetable_data_archive), dir, true)
        .context("! failed to extract zip")?;

    Ok(())
}

enum ProcessingResult {
    Success(u32),
    Skipped(u32),
}

struct JourneyProcessingJob {
    db: Arc<Pool>,
    service: ServiceLeg,
    timetable: Arc<Timetable>,
    footnotes: Arc<Footnotes>,
    companies: Arc<Companies>,
}

impl JourneyProcessingJob {
    pub(crate) async fn process(self, worker_id: usize) -> Result<ProcessingResult> {
        println!(
            "+ Processing service {} with worker {worker_id}",
            self.service.service_identification.0
        );

        let mut db = self
            .db
            .get()
            .await
            .context("worker failed to get client from pool")?;

        let transaction = db
            .transaction()
            .await
            .context("failed to start transaction")?;

        let footnote = if self.service.validity.footnote == 0 {
            &Footnote::always_valid(&self.timetable.identification)
        } else {
            self.footnotes
                .get_by_id(self.service.validity.footnote)
                .context("! footnote not found")?
        };

        let service_number = match self.get_service_number() {
            Some(service_number) => service_number,
            None => {
                return Ok(ProcessingResult::Skipped(
                    self.service.service_identification.0,
                ));
            }
        };

        let company = self
            .companies
            .get_by_id(self.service.service_number.company_number)
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
                self.service.transport_mode.code.clone().into(),
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
            .await
            .context("! failed to insert service(s)")?;

        assert_eq!(inserted_service.len(), 1);

        let service_id: Uuid = inserted_service.first().unwrap().get("id");

        let (journey_attributes, stop_attributes): (Vec<_>, Vec<_>) =
            self.service.attributes.iter().cloned().partition(|attr| {
                attr.first_stop == 1 && attr.last_stop == self.service.num_stops()
            });

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
            .iterate_valid_dates(&self.timetable.identification)
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
                    vec![self.service.service_identification.0.to_string()].into(),
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

            let inserted_journey = transaction
                .query(
                    journey_insert_sql.as_str(),
                    &journey_insert_params.as_params(),
                )
                .await
                .context(format!(
                    "query: {} - params: {:?}",
                    journey_insert_sql.as_str(),
                    journey_insert_params.as_params()
                ))
                .context("! failed to insert journey")?;

            assert_eq!(inserted_journey.len(), 1);

            let journey_id: Uuid = inserted_journey.first().unwrap().get("id");

            for (idx, (event, platform)) in self.service.station_events.iter().enumerate() {
                at_least_one_journey_event = true;

                let stop_attributes = self.service.stop_number(event).and_then(|stop_number| {
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
                .await
                .context(format!("query: {journey_event_insert_query}"))
                .context("! could not insert journey events")?;
        }

        transaction
            .commit()
            .await
            .context("! could not commit transaction")?;

        Ok(ProcessingResult::Success(
            self.service.service_identification.0,
        ))
    }

    fn get_service_number(&self) -> Option<String> {
        (self.service.service_number.service_number != 0)
            .then_some(self.service.service_number.service_number.to_string())
            .or(self.service.service_number.variant.clone())
    }
}

async fn worker(
    id: usize,
    job_rx: async_channel::Receiver<JourneyProcessingJob>,
    result_tx: async_channel::Sender<Result<ProcessingResult>>,
) {
    println!("+ Worker {id} started");
    while let Ok(job) = job_rx.recv().await {
        let output = job.process(id).await;
        let _ = result_tx.send(output).await;
    }
    println!("+ Worker {id} exiting");
}

async fn collect_results(mut rx: async_channel::Receiver<Result<ProcessingResult>>) {
    while let Ok(result) = rx.recv().await {
        match result {
            Ok(result) => match result {
                ProcessingResult::Success(service_number) => {
                    println!("+ Service {service_number} processed successfully")
                }
                ProcessingResult::Skipped(service_number) => {
                    println!("+ Service {service_number} skipped")
                }
            },
            Err(e) => {
                println!("! Failed to process service: {e}")
            }
        }
    }
}

pub async fn import(db: Arc<Pool>, input_path: Option<String>) -> Result<()> {
    let data_dir: PathBuf = if let Some(input_path) = input_path {
        println!("+ Using input path: {}", input_path);
        PathBuf::from(input_path)
    } else {
        println!("+ Downloading and unzipping latest data");
        let data_dir = env::temp_dir().join(format!("kedeng-data-importer-{}", Uuid::new_v4()));
        fs::create_dir_all(&data_dir).context("! failed to create temp dir")?;
        println!("+ Created temp dir: {}", data_dir.display());

        download_and_extract_data(&data_dir).await?;

        data_dir
    };

    let timetable = load_file(&data_dir.join("./timetbls.dat"), timetable_file)?;
    println!("+ Loaded {} services", timetable.data.len());
    let timetable = Arc::new(timetable);
    // println!("{:#?}", timetable);
    // return Ok(());

    let footnotes = load_file(&data_dir.join("./footnote.dat"), footnote_file)?;
    println!("+ Loaded {} footnotes", footnotes.data.len());
    let footnotes = Arc::new(footnotes);

    let companies = load_file(&data_dir.join("./company.dat"), company_file)?;
    println!("+ Loaded {} companies", companies.data.len());
    let companies = Arc::new(companies);

    let (job_tx, job_rx) = async_channel::unbounded::<JourneyProcessingJob>();
    let (result_tx, result_rx) = async_channel::unbounded::<Result<ProcessingResult>>();

    let worker_handles = (0..5)
        .map(|id| tokio::spawn(worker(id, job_rx.clone(), result_tx.clone())))
        .collect::<Vec<_>>();
    drop(result_tx);

    let collector_handle = tokio::spawn(collect_results(result_rx));

    let services = timetable.data.iter().flat_map(Service::split_legs);
    for service in services {
        let job = JourneyProcessingJob {
            db: Arc::clone(&db),
            service,
            timetable: Arc::clone(&timetable),
            footnotes: Arc::clone(&footnotes),
            companies: Arc::clone(&companies),
        };

        if job_tx.send(job).await.is_err() {
            eprintln!("! job receiver has been dropped, aborting");
            break;
        }
    }

    drop(job_tx);

    for handle in worker_handles {
        handle.await?;
    }
    collector_handle.await?;

    println!("+ All done!");

    Ok(())
}
