use actix_web::{web, Responder};

use crate::api::ApiResponse;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::use_cases::get_file_keywords;

pub async fn handler(
    storage: PersonalizedFileStorage,
    query: web::Query<get_file_keywords::Input>,
) -> impl Responder {
    let result: ApiResponse<_> = get_file_keywords::command(&storage, &query).await.into();
    result
}
