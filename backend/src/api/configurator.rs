use crate::api::endpoints::auth::login;
use crate::api::endpoints::auth_registration_complete::registration_complete;
use crate::api::endpoints::auth_registration_verify::registration_verify;
use crate::api::endpoints::registration::registration_init;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/login", web::post().to(login))
            .route("/registration/init", web::post().to(registration_init))
            .route("/registration/verify", web::post().to(registration_verify))
            .route(
                "/registration/complete",
                web::post().to(registration_complete),
            ),
    );
}
