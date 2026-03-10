use crate::api::endpoints::{
    auth_change_password, auth_login, auth_me, auth_registration_complete,
    auth_registration_init, auth_registration_verify, auth_restore_complete, auth_restore_init,
    auth_restore_verify, auth_update_profile,
    chat_create, chat_get, chat_list, chat_models, chat_send_message, ws,
};
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/auth")
            .route("/login", web::post().to(auth_login::handler))
            .route("/me", web::post().to(auth_me::handler))
            .route("/change-profile", web::post().to(auth_update_profile::handler))
            .route("/change-password", web::post().to(auth_change_password::handler))
            .route("/registration/init", web::post().to(auth_registration_init::handler))
            .route("/registration/verify", web::post().to(auth_registration_verify::handler))
            .route(
                "/registration/complete",
                web::post().to(auth_registration_complete::handler),
            )
            .route("/password/restore/init", web::post().to(auth_restore_init::handler))
            .route("/password/restore/verify", web::post().to(auth_restore_verify::handler))
            .route(
                "/password/restore/complete",
                web::post().to(auth_restore_complete::handler),
            ),
    );
    cfg.service(
        web::scope("/api/chat")
            .route("/models", web::get().to(chat_models::handler))
            .route("", web::post().to(chat_create::handler))
            .route("", web::get().to(chat_list::handler))
            .route("/{chat_id}", web::get().to(chat_get::handler))
            .route("/{chat_id}/messages", web::post().to(chat_send_message::handler)),
    );
    cfg.route("/api/ws", web::get().to(ws::handler));
}
