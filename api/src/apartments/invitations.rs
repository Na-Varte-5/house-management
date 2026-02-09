use super::helpers::{ensure_user_has_role, log_property_event};
use super::types::{InvitationRow, InviteRenterPayload, InviteRenterResponse};
use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{
    Apartment, InvitationStatus, NewApartmentRenter, NewRenterInvitation,
    RenterInvitationWithDetails, User,
};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;
use rand::Rng;

fn generate_invitation_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Invite a renter to an apartment by email
///
/// If the email exists in the system, the user is directly assigned as a renter.
/// If the email doesn't exist, creates an invitation that can be accepted when the user registers.
/// Requires Admin, Manager role, or apartment ownership.
#[utoipa::path(
    post,
    path = "/api/v1/apartments/{id}/invite",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    request_body = InviteRenterPayload,
    responses(
        (status = 200, description = "User exists - directly assigned as renter", body = InviteRenterResponse),
        (status = 201, description = "User does not exist - invitation created", body = InviteRenterResponse),
        (status = 403, description = "Forbidden - requires Admin, Manager role, or apartment ownership"),
        (status = 404, description = "Apartment not found"),
        (status = 409, description = "User is already a renter or invitation already pending"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn invite_renter(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<InviteRenterPayload>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;
    use crate::schema::apartments::dsl as apt;
    use crate::schema::renter_invitations::dsl as ri;
    use crate::schema::users::dsl as users;

    let apartment_id = path.into_inner();
    let email = payload.email.trim().to_lowercase();

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    if !is_admin_or_manager {
        let is_owner: bool = ao::apartment_owners
            .filter(
                ao::apartment_id
                    .eq(apartment_id)
                    .and(ao::user_id.eq(current_user_id)),
            )
            .count()
            .get_result::<i64>(&mut conn)?
            > 0;

        if !is_owner {
            return Err(AppError::Forbidden);
        }
    }

    let _apartment: Apartment = apt::apartments
        .filter(apt::id.eq(apartment_id).and(apt::is_deleted.eq(false)))
        .select(Apartment::as_select())
        .first(&mut conn)
        .map_err(|_| AppError::NotFound)?;

    let existing_user: Option<User> = users::users
        .filter(users::email.eq(&email))
        .select(User::as_select())
        .first(&mut conn)
        .ok();

    if let Some(user) = existing_user {
        let already_renter: Result<u64, _> = ar::apartment_renters
            .filter(
                ar::apartment_id
                    .eq(apartment_id)
                    .and(ar::user_id.eq(user.id)),
            )
            .select(ar::id)
            .first(&mut conn);

        if already_renter.is_ok() {
            return Err(AppError::BadRequest(
                "User is already a renter of this apartment".into(),
            ));
        }

        let new_renter = NewApartmentRenter {
            apartment_id,
            user_id: user.id,
            start_date: payload.start_date,
            end_date: payload.end_date,
            is_active: Some(true),
        };

        diesel::insert_into(ar::apartment_renters)
            .values(&new_renter)
            .execute(&mut conn)?;

        ensure_user_has_role(user.id, "Renter", &mut conn).await?;

        log_property_event(
            apartment_id,
            "renter_added",
            Some(user.id),
            current_user_id,
            format!("Added {} as renter (via invite)", user.name),
            Some(
                serde_json::json!({
                    "start_date": payload.start_date.map(|d| d.to_string()),
                    "end_date": payload.end_date.map(|d| d.to_string()),
                    "method": "invite"
                })
                .to_string(),
            ),
            &mut conn,
        )
        .await?;

        return Ok(HttpResponse::Ok().json(InviteRenterResponse {
            invitation_id: 0,
            email: email.clone(),
            status: "assigned".to_string(),
            message: format!("User {} has been assigned as a renter", user.name),
        }));
    }

    let pending_invitation: Option<u64> = ri::renter_invitations
        .filter(
            ri::apartment_id
                .eq(apartment_id)
                .and(ri::email.eq(&email))
                .and(ri::status.eq(InvitationStatus::Pending)),
        )
        .select(ri::id)
        .first(&mut conn)
        .ok();

    if pending_invitation.is_some() {
        return Err(AppError::BadRequest(
            "An invitation is already pending for this email".into(),
        ));
    }

    let token = generate_invitation_token();
    let expires_at = chrono::Utc::now().naive_utc() + chrono::Duration::days(7);

    let new_invitation = NewRenterInvitation {
        apartment_id,
        email: email.clone(),
        token,
        start_date: payload.start_date,
        end_date: payload.end_date,
        invited_by: current_user_id,
        status: InvitationStatus::Pending,
        expires_at,
    };

    diesel::insert_into(ri::renter_invitations)
        .values(&new_invitation)
        .execute(&mut conn)?;

    let invitation_id: u64 = ri::renter_invitations
        .filter(ri::email.eq(&email).and(ri::apartment_id.eq(apartment_id)))
        .order(ri::created_at.desc())
        .select(ri::id)
        .first(&mut conn)?;

    log_property_event(
        apartment_id,
        "renter_invited",
        None,
        current_user_id,
        format!("Sent renter invitation to {}", email),
        Some(
            serde_json::json!({
                "email": email,
                "expires_at": expires_at.to_string(),
            })
            .to_string(),
        ),
        &mut conn,
    )
    .await?;

    Ok(HttpResponse::Created().json(InviteRenterResponse {
        invitation_id,
        email: email.clone(),
        status: "pending".to_string(),
        message: format!(
            "Invitation sent to {}. The user can accept it when they register.",
            email
        ),
    }))
}

/// List pending invitations for an apartment
///
/// Returns all pending renter invitations for the specified apartment.
/// Requires Admin, Manager role, or apartment ownership.
#[utoipa::path(
    get,
    path = "/api/v1/apartments/{id}/invitations",
    params(
        ("id" = u64, Path, description = "Apartment ID")
    ),
    responses(
        (status = 200, description = "List of invitations", body = Vec<RenterInvitationWithDetails>),
        (status = 403, description = "Forbidden - requires Admin, Manager role, or apartment ownership"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn list_apartment_invitations(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartments::dsl as apt;
    use crate::schema::buildings::dsl as bld;
    use crate::schema::renter_invitations::dsl as ri;
    use crate::schema::users::dsl as users;

    let apartment_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    if !is_admin_or_manager {
        let user_id: u64 = auth
            .claims
            .sub
            .parse()
            .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

        let is_owner: bool = ao::apartment_owners
            .filter(
                ao::apartment_id
                    .eq(apartment_id)
                    .and(ao::user_id.eq(user_id)),
            )
            .count()
            .get_result::<i64>(&mut conn)?
            > 0;

        if !is_owner {
            return Err(AppError::Forbidden);
        }
    }

    let (apt_number, building_id): (String, u64) = apt::apartments
        .filter(apt::id.eq(apartment_id))
        .select((apt::number, apt::building_id))
        .first(&mut conn)?;

    let building_address: String = bld::buildings
        .filter(bld::id.eq(building_id))
        .select(bld::address)
        .first(&mut conn)?;

    let invitations_data: Vec<InvitationRow> = ri::renter_invitations
        .filter(ri::apartment_id.eq(apartment_id))
        .select((
            ri::id,
            ri::apartment_id,
            ri::email,
            ri::start_date,
            ri::end_date,
            ri::invited_by,
            ri::status,
            ri::expires_at,
            ri::created_at,
        ))
        .order(ri::created_at.desc())
        .load(&mut conn)?;

    let mut result: Vec<RenterInvitationWithDetails> = Vec::new();
    for (id, apt_id, email, start_date, end_date, invited_by, status, expires_at, created_at) in
        invitations_data
    {
        let invited_by_name: String = users::users
            .filter(users::id.eq(invited_by))
            .select(users::name)
            .first(&mut conn)?;

        result.push(RenterInvitationWithDetails {
            id,
            apartment_id: apt_id,
            apartment_number: apt_number.clone(),
            building_address: building_address.clone(),
            email,
            start_date,
            end_date,
            invited_by,
            invited_by_name,
            status,
            expires_at,
            created_at,
        });
    }

    Ok(HttpResponse::Ok().json(result))
}

/// Cancel a pending invitation
///
/// Cancels a pending renter invitation.
/// Requires Admin, Manager role, or apartment ownership.
#[utoipa::path(
    delete,
    path = "/api/v1/apartments/{id}/invitations/{invitation_id}",
    params(
        ("id" = u64, Path, description = "Apartment ID"),
        ("invitation_id" = u64, Path, description = "Invitation ID")
    ),
    responses(
        (status = 204, description = "Invitation cancelled"),
        (status = 403, description = "Forbidden - requires Admin, Manager role, or apartment ownership"),
        (status = 404, description = "Invitation not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Apartments",
    security(("bearer_auth" = []))
)]
pub async fn cancel_invitation(
    auth: AuthContext,
    path: web::Path<(u64, u64)>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::renter_invitations::dsl as ri;

    let (apartment_id, invitation_id) = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let current_user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;

    let is_admin_or_manager = auth.has_any_role(&["Admin", "Manager"]);

    if !is_admin_or_manager {
        let is_owner: bool = ao::apartment_owners
            .filter(
                ao::apartment_id
                    .eq(apartment_id)
                    .and(ao::user_id.eq(current_user_id)),
            )
            .count()
            .get_result::<i64>(&mut conn)?
            > 0;

        if !is_owner {
            return Err(AppError::Forbidden);
        }
    }

    let email: String = ri::renter_invitations
        .filter(
            ri::id
                .eq(invitation_id)
                .and(ri::apartment_id.eq(apartment_id))
                .and(ri::status.eq(InvitationStatus::Pending)),
        )
        .select(ri::email)
        .first(&mut conn)
        .map_err(|_| AppError::NotFound)?;

    diesel::update(ri::renter_invitations.filter(ri::id.eq(invitation_id)))
        .set(ri::status.eq(InvitationStatus::Cancelled))
        .execute(&mut conn)?;

    log_property_event(
        apartment_id,
        "invitation_cancelled",
        None,
        current_user_id,
        format!("Cancelled renter invitation for {}", email),
        None,
        &mut conn,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}
