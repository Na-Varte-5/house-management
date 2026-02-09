pub mod attachments;
mod comments;
mod handlers;
mod types;

pub use comments::*;
pub use handlers::*;
pub use types::*;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/requests", web::get().to(list_requests))
        .route("/requests", web::post().to(create_request))
        .route("/requests/{id}", web::get().to(get_request))
        .route("/requests/{id}", web::put().to(update_request))
        .route("/requests/{id}/status", web::put().to(update_status))
        .route("/requests/{id}/history", web::get().to(list_history))
        .route("/requests/{id}/assign", web::put().to(assign_request))
        .route("/requests/{id}/assign", web::delete().to(unassign_request))
        .route("/requests/{id}/escalate", web::post().to(escalate_request))
        // attachment endpoints
        .route(
            "/requests/{id}/attachments",
            web::post().to(attachments::upload_attachment),
        )
        .route(
            "/requests/{id}/attachments",
            web::get().to(attachments::list_attachments),
        )
        .route(
            "/requests/{id}/attachments/deleted",
            web::get().to(attachments::list_deleted_attachments),
        )
        .route(
            "/requests/{id}/attachments/{att_id}",
            web::get().to(attachments::get_attachment_metadata),
        )
        .route(
            "/requests/{id}/attachments/{att_id}/download",
            web::get().to(attachments::download_attachment),
        )
        .route(
            "/requests/{id}/attachments/{att_id}",
            web::delete().to(attachments::delete_attachment),
        )
        .route(
            "/requests/{id}/attachments/{att_id}/restore",
            web::post().to(attachments::restore_attachment),
        )
        // comment endpoints
        .route("/requests/{id}/comments", web::get().to(list_comments))
        .route("/requests/{id}/comments", web::post().to(create_comment))
        .route(
            "/requests/{id}/comments/{comment_id}",
            web::delete().to(delete_comment),
        );
}
