use crate::importers::{station_geometry, stations, timetable};
use clap::{Parser, Subcommand};
use opentelemetry::global;
use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::SdkMeterProvider;

mod db;
mod importers;
mod util;

#[derive(Parser)]
#[command(
    name = "OV plan data importer",
    about = "Import OV plan data from various sources (NS timetable, station data from NS API,..."
)]
struct Cli {
    #[command(subcommand)]
    importer: Importer,

    #[arg(short = 'H', long, env = "DB_HOST")]
    db_host: String,

    #[arg(short = 'P', long, env = "DB_PORT", default_value = "5432")]
    db_port: u16,

    #[arg(short = 'u', long, env = "DB_USER")]
    db_user: String,

    #[arg(short = 'p', long, env = "DB_PASSWORD")]
    db_password: String,

    #[arg(short = 'n', long, env = "DB_NAME")]
    db_name: String,
}

#[derive(Subcommand)]
enum Importer {
    Timetable {
        #[arg(short, long)]
        input_path: Option<String>,
    },

    Stations {
        #[arg(short = 'k', long, env = "NS_API_KEY")]
        api_key: String,
    },

    StationGeometry {
        #[arg(short = 'k', long, env = "NS_API_KEY")]
        api_key: String,
    },
}

fn init_metrics_provider() -> anyhow::Result<()> {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_protocol(Protocol::Grpc)
        .with_endpoint("http://localhost:4317")
        .build()?;

    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("kedeng/date-importer")
                .build(),
        )
        .build();

    global::set_meter_provider(provider.clone());

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut db = db::connect(
        cli.db_user.as_str(),
        cli.db_password.as_str(),
        cli.db_name.as_str(),
        cli.db_host.as_str(),
        Some(cli.db_port),
    );

    // init_metrics_provider()?;

    match cli.importer {
        Importer::Timetable { input_path } => timetable::import(&mut db, input_path)?,
        Importer::Stations { api_key } => stations::import(&mut db, api_key.as_str())?,
        Importer::StationGeometry { api_key } => {
            station_geometry::import(&mut db, api_key.as_str())?
        }
    };

    db.close()?;

    Ok(())
}
