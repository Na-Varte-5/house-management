use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "House Management API",
        version = "1.0.0",
        description = "API for managing residential properties (Homeowners Associations)\n\n\
                      ## Authentication\n\
                      Most endpoints require JWT authentication. Obtain a token by calling `/api/v1/login`.\n\n\
                      ## Roles\n\
                      - **Admin**: Full system access\n\
                      - **Manager**: Manage properties, maintenance, proposals\n\
                      - **Homeowner**: View owned properties, submit maintenance requests, vote\n\
                      - **Renter**: View rented properties, submit maintenance requests\n\
                      - **HOA Member**: Participate in community activities",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT"
        )
    ),
    paths(
        // Authentication
        crate::auth::handlers::register,
        crate::auth::handlers::login,

        // Buildings
        crate::buildings::list_buildings,
        crate::buildings::get_building,
        crate::buildings::create_building,
        crate::buildings::list_my_buildings,
        crate::buildings::delete_building,
        crate::buildings::list_deleted_buildings,
        crate::buildings::restore_building,

        // Apartments
        crate::apartments::list_apartments,
        crate::apartments::list_building_apartments,
        crate::apartments::list_my_building_apartments,
        crate::apartments::list_my_apartments,
        crate::apartments::create_apartment,
        crate::apartments::delete_apartment,
        crate::apartments::list_deleted_apartments,
        crate::apartments::restore_apartment,
        crate::apartments::list_apartment_owners,
        crate::apartments::add_apartment_owner,
        crate::apartments::remove_apartment_owner,

        // Voting
        crate::voting::list_proposals,
        crate::voting::get_proposal,
        crate::voting::create_proposal,
        crate::voting::cast_vote,
        crate::voting::tally_results,

        // Maintenance
        crate::maintenance::list_requests,
        crate::maintenance::get_request,
        crate::maintenance::create_request,
        crate::maintenance::update_request,
        crate::maintenance::update_status,
        crate::maintenance::list_history,
        crate::maintenance::assign_request,
        crate::maintenance::unassign_request,
        crate::maintenance::attachments::upload_attachment,
        crate::maintenance::attachments::list_attachments,
        crate::maintenance::attachments::list_deleted_attachments,
        crate::maintenance::attachments::get_attachment_metadata,
        crate::maintenance::attachments::download_attachment,
        crate::maintenance::attachments::delete_attachment,
        crate::maintenance::attachments::restore_attachment,

        // Announcements
        crate::announcements::list_public,
        crate::announcements::list_auth,
        crate::announcements::get_one,
        crate::announcements::create,
        crate::announcements::update,
        crate::announcements::delete_soft,
        crate::announcements::restore,
        crate::announcements::toggle_pin,
        crate::announcements::list_comments,
        crate::announcements::create_comment,
        crate::announcements::delete_comment,
        crate::announcements::restore_comment,
        crate::announcements::list_deleted,
        crate::announcements::purge,
        crate::announcements::publish_now,
        crate::announcements::purge_comment,

        // Users
        crate::users::list_users,
        crate::users::create_user,
        crate::users::list_users_with_roles,
        crate::users::set_user_roles,
        crate::users::list_public_users,
    ),
    components(
        schemas(
            // Auth types
            crate::auth::types::RegisterRequest,
            crate::auth::types::LoginRequest,
            crate::auth::types::AuthResponse,
            crate::auth::types::Claims,

            // Models
            crate::models::User,
            crate::models::Building,
            crate::models::NewBuilding,
            crate::models::Apartment,
            crate::models::NewApartment,
            crate::models::Proposal,
            crate::models::NewProposal,
            crate::models::Vote,
            crate::models::ProposalResult,
            crate::models::VotingMethod,
            crate::models::VoteChoice,
            crate::models::ProposalStatus,
            crate::models::MaintenanceRequest,
            crate::models::NewMaintenanceRequest,
            crate::models::MaintenanceRequestAttachment,
            crate::models::MaintenanceRequestHistory,

            // Apartment-specific types
            crate::apartments::OwnerAssignPayload,
            crate::apartments::ApartmentWithBuilding,

            // Voting-specific types
            crate::voting::ProposalWithVotes,
            crate::voting::CreateProposalPayload,
            crate::voting::CastVotePayload,

            // Maintenance-specific types
            crate::maintenance::MaintenanceRequestEnriched,
            crate::maintenance::StatusUpdatePayload,
            crate::maintenance::UpdateRequestPayload,
            crate::maintenance::AssignPayload,

            // Announcements types
            crate::models::AnnouncementComment,
            crate::announcements::CreateAnnouncementRequest,
            crate::announcements::UpdateAnnouncementRequest,
            crate::announcements::CreateCommentRequest,
            crate::announcements::AnnouncementOut,
            crate::announcements::CommentOut,

            // Users types
            crate::users::SetRolesRequest,
            crate::users::UserRolesResponse,
            crate::users::UserWithRoles,

            // Pagination types
            crate::pagination::PaginationMeta,
        )
    ),
    tags(
        (name = "Authentication", description = "User registration and login"),
        (name = "Buildings", description = "Building management (Admin/Manager)"),
        (name = "Apartments", description = "Apartment management"),
        (name = "Voting", description = "Proposals and voting system"),
        (name = "Maintenance", description = "Maintenance request management"),
        (name = "Announcements", description = "Community announcements"),
        (name = "Users", description = "User management (Admin only)"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some(
                            "JWT token obtained from `/api/v1/login` endpoint. \
                                          Include in Authorization header as: `Bearer <token>`",
                        ))
                        .build(),
                ),
            );
        }
    }
}
