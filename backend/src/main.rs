mod api;
mod config;
mod models;
mod providers;
mod use_cases;

#[cfg(test)]
mod tests;

use actix_web::web;
use clap::Parser;
use config::Config;
use providers::{LocalFileSystemProvider, S3Provider, SMTPProvider, SQLiteProvider};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "backend")]
#[command(about = "Backend API server")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    Run {
        #[arg(long)]
        config: PathBuf,
    },
    DownloadSqlite {
        #[arg(long)]
        config: PathBuf,
    },
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { config } => run_server(config).await,
        Commands::DownloadSqlite { config } => download_sqlite(config).await,
    }
}

async fn run_server(config_path: PathBuf) -> std::io::Result<()> {
    let config = Config::from_file(&config_path)
        .expect("Failed to load configuration file");

    let s3_provider = S3Provider::new(
        config.s3.bucket.clone(),
        config.s3.region.clone(),
        config.s3.client_id.clone(),
        config.s3.client_secret.clone(),
        config.s3.host.clone(),
    )
    .await
    .expect("Failed to initialize S3 provider");

    let fs_provider = LocalFileSystemProvider::new(
        PathBuf::from(config.local_fs.root_path.clone())
    )
    .expect("Failed to initialize local filesystem provider");

    let sqlite_provider = SQLiteProvider::new(
        Arc::new(s3_provider),
        Arc::new(fs_provider),
        &config.sqlite.s3_object_path,
    )
    .await
    .expect("Failed to initialize SQLite provider");

    let _smtp_provider = SMTPProvider::new(
        &config.smtp.host,
        config.smtp.port,
        config.smtp.use_tls,
        &config.smtp.username,
        &config.smtp.password,
        "noreply@example.com",
    )
    .expect("Failed to initialize SMTP provider");

    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable must be set");

    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(web::Data::new(Arc::new(sqlite_provider.clone())))
            .app_data(web::Data::new(jwt_secret.clone()))
            .configure(api::configure_routes)
    })
    .bind((config.addr.as_str(), config.port))?
    .run();

    println!("Backend server starting on http://{}:{}", config.addr, config.port);
    server.await
}

async fn download_sqlite(config_path: PathBuf) -> std::io::Result<()> {
    let config = Config::from_file(&config_path)
        .expect("Failed to load configuration file");

    let s3_provider = S3Provider::new(
        config.s3.bucket.clone(),
        config.s3.region.clone(),
        config.s3.client_id.clone(),
        config.s3.client_secret.clone(),
        config.s3.host.clone(),
    )
    .await
    .expect("Failed to initialize S3 provider");

    let fs_provider = LocalFileSystemProvider::new(
        PathBuf::from(config.local_fs.root_path.clone())
    )
    .expect("Failed to initialize local filesystem provider");

    if s3_provider.object_exists(&config.sqlite.s3_object_path).await.unwrap_or(false) {
        let data = s3_provider.get_object(&config.sqlite.s3_object_path)
            .await
            .expect("Failed to download SQLite database");
        
        let path = fs_provider.save(data)
            .expect("Failed to save SQLite database locally");
        
        println!("SQLite database downloaded to: {:?}", path);
    } else {
        println!("SQLite database does not exist in S3");
    }

    Ok(())
}