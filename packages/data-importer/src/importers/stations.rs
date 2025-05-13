use crate::db;
use anyhow::Result;
use postgres::Client;
use sea_query::{Expr, PostgresQueryBuilder, Query, QueryStatementWriter, SimpleExpr, ValueTuple};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Deserialize, Clone)]
struct StationIds {
    #[serde(rename = "uicCode")]
    pub uic: String,
    #[serde(rename = "uicCdCode")]
    pub uic_cd: String,
    #[serde(rename = "cdCode")]
    pub cd: Option<usize>,
    #[serde(rename = "evaCode")]
    pub eva: String,
    pub code: String,
}

#[derive(Debug, Deserialize, Clone)]
struct StationNames {
    pub long: String,
    pub medium: String,
    pub short: String,
    pub synonyms: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
enum StationType {
    #[serde(rename = "MEGA_STATION")]
    MegaStation,
    #[serde(rename = "INTERCITY_HUB_STATION")]
    IntercityHub,
    #[serde(rename = "INTERCITY_STATION")]
    Intercity,
    #[serde(rename = "EXPRESS_TRAIN_HUB_STATION")]
    ExpressHub,
    #[serde(rename = "EXPRESS_TRAIN_STATION")]
    Express,
    #[serde(rename = "LOCAL_TRAIN_STATION")]
    Local,
    #[serde(rename = "LOCAL_TRAIN_HUB_STATION")]
    LocalHub,
    #[serde(rename = "OPTIONAL_STATION")]
    Optional,
}

impl Display for StationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StationType::MegaStation => f.write_str("MEGA_STATION"),
            StationType::IntercityHub => f.write_str("INTERCITY_HUB_STATION"),
            StationType::Intercity => f.write_str("INTERCITY_STATION"),
            StationType::ExpressHub => f.write_str("EXPRESS_TRAIN_HUB_STATION"),
            StationType::Express => f.write_str("EXPRESS_TRAIN_STATION"),
            StationType::Local => f.write_str("LOCAL_TRAIN_STATION"),
            StationType::LocalHub => f.write_str("LOCAL_TRAIN_HUB_STATION"),
            StationType::Optional => f.write_str("OPTIONAL_STATION"),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct StationFeatures {
    pub has_travel_assistance: bool,
    pub is_border_stop: bool,
    #[serde(rename = "availableForAccessibleTravel")]
    pub is_available_for_accessible_travel: bool,
    pub has_known_facilities: bool,
    pub are_tracks_independently_accessible: bool,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Station {
    pub id: StationIds,
    pub names: StationNames,
    pub station_type: StationType,
    pub country: String,
    pub tracks: Vec<String>,
    #[serde(flatten)]
    pub features: StationFeatures,
    pub location: Location,
}

#[derive(Debug, Deserialize)]
struct StationResponse {
    pub payload: Vec<Station>,
}

const API_URL: &str = "https://gateway.apiportal.ns.nl/nsapp-stations/v3";

pub fn import(db: &mut Client, api_key: &str) -> Result<()> {
    let response = ureq::get(API_URL)
        .header("Ocp-Apim-Subscription-Key", api_key)
        .call()?
        .body_mut()
        .read_json::<StationResponse>()?;

    let missing_data = include_str!("./stations/missing.json");
    let missing_data = serde_json::from_str::<StationResponse>(missing_data)?;

    let mut qb = Query::insert();
    qb.into_table(db::Station::Table).columns([
        db::Station::UicCode,
        db::Station::UicCdCode,
        db::Station::EvaCode,
        db::Station::CdCode,
        db::Station::Code,
        db::Station::StationType,
        db::Station::NameLong,
        db::Station::NameMedium,
        db::Station::NameShort,
        db::Station::NameSynonyms,
        db::Station::Country,
        db::Station::Tracks,
        db::Station::HasTravelAssistance,
        db::Station::IsBorderStop,
        db::Station::IsAvailableForAccessibleTravel,
        db::Station::HasKnownFacilities,
        db::Station::AreTracksIndependentlyAccessible,
        db::Station::Location,
    ]);

    for station in response.payload.iter().chain(missing_data.payload.iter()) {
        qb.values([
            station.id.uic.clone().into(),
            station.id.uic_cd.clone().into(),
            station.id.eva.clone().into(),
            station.id.cd.map(|_| station.id.cd.unwrap() as u64).into(),
            station.id.code.to_lowercase().into(),
            station.station_type.to_string().into(),
            station.names.long.clone().into(),
            station.names.medium.clone().into(),
            station.names.short.clone().into(),
            (if station.names.synonyms.is_empty() {
                None
            } else {
                Some(station.names.synonyms.clone())
            })
            .into(),
            station.country.clone().into(),
            (if station.tracks.is_empty() {
                None
            } else {
                Some(station.tracks.clone())
            })
            .into(),
            station.features.has_travel_assistance.into(),
            station.features.is_border_stop.into(),
            station.features.is_available_for_accessible_travel.into(),
            station.features.has_known_facilities.into(),
            station.features.are_tracks_independently_accessible.into(),
            Expr::cust(format!(
                "point({}, {})",
                station.location.lat, station.location.lng
            )),
        ])?;
    }

    let sql = qb.to_string(PostgresQueryBuilder);
    println!("{}", sql);

    db.batch_execute(&sql)?;

    Ok(())
}
