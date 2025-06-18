use crate::importers::{station_geometry, stations, timetable};
use clap::{Parser, Subcommand};
use std::sync::Arc;

mod db;
pub mod importers;
mod ns;
pub(crate) mod util;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let db = db::connect_async(
        cli.db_user.as_str(),
        cli.db_password.as_str(),
        cli.db_name.as_str(),
        cli.db_host.as_str(),
        Some(cli.db_port),
    )
    .await;

    let db = Arc::new(db);

    // init_metrics_provider()?;

    match cli.importer {
        Importer::Timetable { input_path } => timetable::import(db, input_path).await?,
        Importer::Stations { api_key } => stations::import(db, api_key.as_str()).await?,
        Importer::StationGeometry { api_key } => {
            station_geometry::import(db, api_key.as_str()).await?
        }
    };

    Ok(())
}
