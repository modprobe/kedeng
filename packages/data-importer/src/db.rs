use postgres::Client;
use postgres::Config;
use postgres::NoTls;
use sea_query::Iden;

pub fn connect(
    username: &str,
    password: &str,
    database: &str,
    host: &str,
    port: Option<u16>,
) -> Client {
    let mut config = Config::new();
    config
        .user(username)
        .password(password)
        .dbname(database)
        .host(host)
        .port(port.unwrap_or(5432));

    config.connect(NoTls).expect("failed to connect")
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
