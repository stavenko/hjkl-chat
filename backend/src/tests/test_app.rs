use crate::api;
use crate::config::Config;
use crate::providers::smtp::SMTPProvider;
use crate::providers::sqlite::SQLiteProvider;
use crate::providers::{LocalFileSystemProvider, S3Provider};
use crate::use_cases::registration::RegistrationUseCase;
use crate::use_cases::{RegistrationCompleteUseCase, RegistrationVerifyUseCase};
use actix_web::{web, App};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

static CONFIG_PATH_ENV_VAR: &str = "HJKL_CHAT_BACKEND_CONFIG_PATH";
static TEST_JWT_SECRET: &str = "test-secret-key-for-jwt-signing";

#[allow(dead_code)]
pub struct AppDetails {
    pub sqlite: Arc<SQLiteProvider>,
    pub config: Config,
}

pub async fn create_app_with_fixtures<Fut, Out>(
    fixtures_runner: impl FnOnce(AppDetails) -> Fut,
) -> anyhow::Result<(
    AppDetails,
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
    >,
    Out,
)>
where
    Fut: std::future::Future<Output = Out>,
{
    let config_path = std::env::var(CONFIG_PATH_ENV_VAR)
        .unwrap_or_else(|_| "docker/test/config.toml".to_string());
    let mut config = Config::from_file(std::path::Path::new(&config_path))
        .expect("Failed to load test config");

    let test_id = Uuid::new_v4().simple().to_string();
    config.s3.bucket = format!("{}-{}", config.s3.bucket, test_id);
    config.local_fs.root_path = format!("/tmp/test-local-fs-{}", test_id);
    config.sqlite.s3_object_path = format!("test-{}.db", test_id);

    let s3_provider = Arc::new(
        S3Provider::new(
            config.s3.bucket.clone(),
            config.s3.region.clone(),
            config.s3.client_id.clone(),
            config.s3.client_secret.clone(),
            config.s3.host.clone(),
        )
        .await
        .expect("Failed to initialize S3 provider"),
    );

    s3_provider
        .client
        .create_bucket()
        .bucket(&config.s3.bucket)
        .send()
        .await
        .expect("Failed to create test S3 bucket");

    let fs_provider = LocalFileSystemProvider::new(PathBuf::from(&config.local_fs.root_path))
        .expect("Failed to initialize local filesystem provider");

    let sqlite_provider = SQLiteProvider::new(
        s3_provider.clone(),
        Arc::new(fs_provider),
        &config.sqlite.s3_object_path,
    )
    .await
    .expect("Failed to initialize SQLite provider");

    let sqlite_provider = Arc::new(sqlite_provider);

    let smtp_provider = Arc::new(
        SMTPProvider::new(
            &config.smtp.host,
            config.smtp.port,
            config.smtp.use_tls,
            &config.smtp.username,
            &config.smtp.password,
            &config.smtp.from_email,
        )
        .expect("Failed to initialize SMTP provider"),
    );

    let jwt_secret = TEST_JWT_SECRET.to_string();

    let registration_use_case = Arc::new(RegistrationUseCase::<SMTPProvider>::new(
        sqlite_provider.clone(),
        smtp_provider.clone(),
    ));
    let registration_verify_use_case =
        Arc::new(RegistrationVerifyUseCase::new(sqlite_provider.clone()));
    let registration_complete_use_case = Arc::new(RegistrationCompleteUseCase::new(
        sqlite_provider.clone(),
        jwt_secret.clone(),
    ));

    let fixture_details = AppDetails {
        sqlite: sqlite_provider.clone(),
        config: config.clone(),
    };

    let fixture_output = fixtures_runner(fixture_details).await;

    let app_details = AppDetails {
        sqlite: sqlite_provider.clone(),
        config: config.clone(),
    };

    let app = actix_test::init_service(
        App::new()
            .app_data(web::Data::new(sqlite_provider.clone()))
            .app_data(web::Data::new(jwt_secret.clone()))
            .app_data(web::Data::new(registration_use_case.clone()))
            .app_data(web::Data::new(registration_verify_use_case.clone()))
            .app_data(web::Data::new(registration_complete_use_case.clone()))
            .configure(api::configure_routes),
    )
    .await;

    Ok((app_details, app, fixture_output))
}

pub async fn create_app() -> anyhow::Result<(
    AppDetails,
    impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
    >,
)> {
    let (details, app, _) = create_app_with_fixtures(|_| async { () }).await?;
    Ok((details, app))
}
