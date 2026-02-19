use actix_web::web;
use crate::collector::api::handlers;

/// Configures and returns the API routes scope
pub fn configure_routes() -> actix_web::Scope {
    web::scope("/api")
        .route("/status", web::get().to(handlers::get_status))
        .route("/messages", web::get().to(handlers::get_messages))
        .route("/publish", web::post().to(handlers::publish_message))
}

