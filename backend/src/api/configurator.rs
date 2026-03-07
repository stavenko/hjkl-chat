use crate::api::endpoints::auth::login;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/auth").route("/login", web::post().to(login)));
}
