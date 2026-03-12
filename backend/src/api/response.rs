use actix_web::body::EitherBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Error {
    pub code: String,
    pub message: String,
}

pub enum ApiResponse<T> {
    Ok(T),
    Err(Error),
}

impl<T> Responder for ApiResponse<T>
where
    T: Serialize + fmt::Debug,
{
    type Body = EitherBody<String>;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let response = match self {
            Self::Ok(r) => serde_json::to_value(&r).map_or_else(
                |_| {
                    tracing::error!("Cannot serialize response: {:?}", r);
                    HttpResponse::InternalServerError()
                        .message_body("Failed to serialize response".to_owned())
                },
                |mut v| {
                    if let serde_json::Value::Object(ref mut map) = v {
                        map.insert("status".to_string(), json!("ok"));
                    }

                    HttpResponse::Ok()
                        .content_type(mime::APPLICATION_JSON)
                        .message_body(serde_json::to_string(&v).unwrap())
                },
            ),
            Self::Err(e) => {
                let status = match e.code.as_str() {
                    "InvalidCredentials" | "SessionNotFound" | "ExpiredSession" => StatusCode::UNAUTHORIZED,
                    "InternalServerError" => StatusCode::INTERNAL_SERVER_ERROR,
                    "VersionConflict" => StatusCode::CONFLICT,
                    _ => StatusCode::BAD_REQUEST,
                };

                let response = json!({
                    "code": e.code,
                    "message": e.message,
                });

                serde_json::to_string(&response)
                    .map_err(|err| err.into())
                    .and_then(|body| {
                        HttpResponse::build(status)
                            .content_type(mime::APPLICATION_JSON)
                            .message_body(body)
                    })
            }
        };
        match response {
            Ok(res) => res.map_into_left_body(),
            Err(err) => HttpResponse::from_error(err).map_into_right_body(),
        }
    }
}

impl<T, E> From<Result<T, E>> for ApiResponse<T>
where
    E: Into<Error>,
{
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(e) => ApiResponse::Ok(e),
            Err(e) => ApiResponse::Err(e.into()),
        }
    }
}
