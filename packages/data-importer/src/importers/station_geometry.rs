use crate::db::StationGeometry;
use crate::ns::create_ns_api_client;
use deadpool_postgres::Pool;
use sea_query::{OnConflict, PostgresQueryBuilder, Query};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

pub async fn import(db_pool: Arc<Pool>, api_key: &str) -> anyhow::Result<()> {
    let db = db_pool.get().await?;
    let http_client = create_ns_api_client(api_key)?;

    let response = http_client
        .get(API_URL)
        .send()
        .await?
        .json::<StationGeometryResponse>()
        .await?;

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
    db.batch_execute(&sql).await?;

    Ok(())
}
