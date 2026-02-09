mod handlers;
mod types;

pub use handlers::*;
pub use types::*;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/proposals", web::get().to(list_proposals))
        .route("/proposals", web::post().to(create_proposal))
        .route("/proposals/{id}", web::get().to(get_proposal))
        .route("/proposals/{id}/vote", web::post().to(cast_vote))
        .route("/proposals/{id}/tally", web::post().to(tally_results));
}
