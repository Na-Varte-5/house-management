mod handlers;
mod helpers;
mod invitations;
mod renters;
mod types;

pub use handlers::*;
pub use invitations::*;
pub use renters::*;
pub use types::*;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/apartments", web::get().to(list_apartments))
        .route("/apartments", web::post().to(create_apartment))
        .route("/apartments/my", web::get().to(list_my_apartments))
        .route(
            "/apartments/deleted",
            web::get().to(list_deleted_apartments),
        )
        .route(
            "/apartments/{id}/restore",
            web::post().to(restore_apartment),
        )
        .route(
            "/buildings/{id}/apartments",
            web::get().to(list_building_apartments),
        )
        .route(
            "/buildings/{id}/apartments/my",
            web::get().to(list_my_building_apartments),
        )
        .route(
            "/apartments/{id}/owners",
            web::get().to(list_apartment_owners),
        )
        .route(
            "/apartments/{id}/owners",
            web::post().to(add_apartment_owner),
        )
        .route(
            "/apartments/{id}/owners/{user_id}",
            web::delete().to(remove_apartment_owner),
        )
        .route(
            "/apartments/{id}/renters",
            web::get().to(list_apartment_renters),
        )
        .route(
            "/apartments/{id}/renters",
            web::post().to(add_apartment_renter),
        )
        .route(
            "/apartments/{id}/renters/{user_id}",
            web::put().to(update_apartment_renter),
        )
        .route(
            "/apartments/{id}/renters/{user_id}",
            web::delete().to(remove_apartment_renter),
        )
        .route(
            "/apartments/{id}/history",
            web::get().to(get_apartment_history),
        )
        .route("/apartments/{id}/invite", web::post().to(invite_renter))
        .route(
            "/apartments/{id}/invitations",
            web::get().to(list_apartment_invitations),
        )
        .route(
            "/apartments/{id}/invitations/{invitation_id}",
            web::delete().to(cancel_invitation),
        )
        .route(
            "/apartments/{id}/permissions",
            web::get().to(get_apartment_permissions),
        )
        .route("/apartments/{id}", web::get().to(get_apartment))
        .route("/apartments/{id}", web::delete().to(delete_apartment));
}
