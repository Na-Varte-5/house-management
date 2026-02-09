mod comments;
mod handlers;
mod helpers;
mod types;

pub use comments::*;
pub use handlers::*;
pub use types::*;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/announcements")
            .route("/public", web::get().to(list_public))
            .route("/deleted", web::get().to(list_deleted))
            .route("", web::get().to(list_auth))
            .route("", web::post().to(create))
            .route("/{id}", web::get().to(get_one))
            .route("/{id}", web::put().to(update))
            .route("/{id}", web::delete().to(delete_soft))
            .route("/{id}/restore", web::post().to(restore))
            .route("/{id}/pin", web::post().to(toggle_pin))
            .route("/{id}/comments", web::get().to(list_comments))
            .route("/{id}/comments", web::post().to(create_comment))
            .route("/comments/{comment_id}", web::delete().to(delete_comment))
            .route(
                "/comments/{comment_id}/restore",
                web::post().to(restore_comment),
            )
            .route(
                "/comments/{comment_id}/purge",
                web::delete().to(purge_comment),
            )
            .route("/{id}/purge", web::delete().to(purge))
            .route("/{id}/publish", web::post().to(publish_now)),
    );
}
