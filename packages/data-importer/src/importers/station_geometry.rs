use crate::db::StationGeometry;
use postgres::Client;
use sea_query::{OnConflict, PostgresQueryBuilder, Query};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LineString {
    r#type: String,
    coordinates: Vec<(f64, f64)>,
}

#[derive(Clone, Debug, Deserialize)]
struct Properties {
    from: String,
    to: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Feature {
    r#type: String,
    geometry: LineString,
    properties: Properties,
}

#[derive(Clone, Debug, Deserialize)]
struct FeatureCollection {
    r#type: String,
    features: Vec<Feature>,
}

#[derive(Clone, Debug, Deserialize)]
struct StationGeometryResponse {
    payload: FeatureCollection,
}

const API_URL: &str = "https://gateway.apiportal.ns.nl/Spoorkaart-API/api/v1/spoorkaart";

pub fn import(db: &mut Client, api_key: &str) -> anyhow::Result<()> {
    let response = ureq::get(API_URL)
        .header("Ocp-Apim-Subscription-Key", api_key)
        .call()?
        .body_mut()
        .read_json::<StationGeometryResponse>()?;

    let mut qb = Query::insert();
    qb.into_table(StationGeometry::Table)
        .columns([
            StationGeometry::From,
            StationGeometry::To,
            StationGeometry::LineString,
        ])
        .on_conflict(
            OnConflict::columns([StationGeometry::From, StationGeometry::To])
                .update_column(StationGeometry::LineString)
                .to_owned(),
        );

    for feature in response.payload.features {
        qb.values_panic([
            feature.properties.from.into(),
            feature.properties.to.into(),
            serde_json::to_string(&feature.geometry)?.into(),
        ]);
    }

    let sql = qb.to_string(PostgresQueryBuilder);
    db.batch_execute(&sql)?;

    Ok(())
}
