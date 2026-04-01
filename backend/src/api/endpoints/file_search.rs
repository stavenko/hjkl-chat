use actix_web::{web, Responder};

use crate::api::ApiResponse;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::use_cases::search_files;

pub async fn handler(
    storage: PersonalizedFileStorage,
    body: web::Json<search_files::Input>,
) -> impl Responder {
    let result: ApiResponse<_> = search_files::command(&storage, &body).await.into();
    result
}
