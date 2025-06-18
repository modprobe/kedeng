use deadpool_postgres::{Pool, PoolConfig, Runtime, Timeouts};
use sea_query::Iden;
use tokio_postgres::NoTls;

pub async fn connect_async(
    username: &str,
    password: &str,
    database: &str,
    host: &str,
    port: Option<u16>,
) -> Pool {
    let mut config = deadpool_postgres::Config::new();
    config.host = Some(host.to_string());
    config.user = Some(username.to_string());
    config.password = Some(password.to_string());
    config.dbname = Some(database.to_string());
    config.port = port;

    config.pool = config.pool.or(Some(PoolConfig::default()));
    config.pool.unwrap().max_size = 10;
    config.pool.unwrap().timeouts = Timeouts::wait_millis(10_000);

    config.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
}

#[derive(Iden)]
pub enum Service {
    Table,
    Id,
    TrainNumber,
    TimetableYear,
    Type,
    Provider,
}

#[derive(Iden)]
pub enum Journey {
    Table,
    Id,
    ServiceId,
    RunningOn,
    Attributes,
    SourceIds,
}

#[derive(Iden)]
pub enum JourneyEvent {
    Table,
    Id,
    JourneyId,
    Station,
    EventTypePlanned,
    StopOrder,
    ArrivalTimePlanned,
    ArrivalPlatformPlanned,
    DepartureTimePlanned,
    DeparturePlatformPlanned,
    Attributes,
}

#[derive(Iden)]
pub enum Station {
    Table,
    UicCode,
    UicCdCode,
    EvaCode,
    CdCode,
    Code,
    StationType,
    NameLong,
    NameMedium,
    NameShort,
    NameSynonyms,
    Country,
    Tracks,
    HasTravelAssistance,
    IsBorderStop,
    IsAvailableForAccessibleTravel,
    HasKnownFacilities,
    AreTracksIndependentlyAccessible,
    Location,
}

#[derive(Iden)]
pub enum StationGeometry {
    Table,
    From,
    To,
    LineString,
}
