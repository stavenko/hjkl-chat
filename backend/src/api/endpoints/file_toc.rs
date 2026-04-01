use actix_web::Responder;

use crate::api::ApiResponse;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::use_cases::get_toc;

pub async fn handler(
    storage: PersonalizedFileStorage,
) -> impl Responder {
    let result: ApiResponse<_> = get_toc::command(&storage).await.into();
    result
}
