use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{InvitationStatus, NewApartmentRenter, RenterInvitationWithDetails};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;

type InvitationRow = (
    u64,
    u64,
    String,
    Option<chrono::NaiveDate>,
    Option<chrono::NaiveDate>,
    u64,
    InvitationStatus,
    chrono::NaiveDateTime,
    Option<chrono::NaiveDateTime>,
);

async fn ensure_user_has_role(
    user_id: u64,
    role_name: &str,
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::MysqlConnection>,
    >,
) -> Result<(), AppError> {
    use crate::schema::roles::dsl as roles_schema;
    use crate::schema::user_roles::dsl as ur_schema;

    let role_id_res: Result<u64, _> = roles_schema::roles
        .filter(roles_schema::name.eq(role_name))
        .select(roles_schema::id)
        .first(conn);

    let role_id = match role_id_res {
        Ok(id) => id,
        Err(_) => {
            diesel::insert_into(roles_schema::roles)
                .values(roles_schema::name.eq(role_name))
                .execute(conn)?;
            roles_schema::roles
                .filter(roles_schema::name.eq(role_name))
                .select(roles_schema::id)
                .first(conn)?
        }
    };

    let exists: Result<(u64, u64), _> = ur_schema::user_roles
        .filter(
            ur_schema::user_id
                .eq(user_id)
                .and(ur_schema::role_id.eq(role_id)),
        )
        .select((ur_schema::user_id, ur_schema::role_id))
        .first(conn);

    if exists.is_err() {
        diesel::insert_into(ur_schema::user_roles)
            .values((
                ur_schema::user_id.eq(user_id),
                ur_schema::role_id.eq(role_id),
            ))
            .execute(conn)?;
    }

    Ok(())
}

#[derive(Serialize, ToSchema)]
pub struct InvitationInfo {
    pub id: u64,
    pub apartment_number: String,
    pub building_address: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub invited_by_name: String,
    pub expires_at: chrono::NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct AcceptInvitationResponse {
    pub success: bool,
    pub message: String,
    pub apartment_id: u64,
    pub apartment_number: String,
    pub building_address: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/invitations/{token}",
    params(
        ("token" = String, Path, description = "Invitation token")
    ),
    responses(
        (status = 200, description = "Invitation details", body = InvitationInfo),
        (status = 404, description = "Invitation not found or expired"),
        (status = 410, description = "Invitation already used or cancelled")
    ),
    tag = "Invitations"
)]
pub async fn get_invitation(
    path: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl as apt;
    use crate::schema::buildings::dsl as bld;
    use crate::schema::renter_invitations::dsl as ri;
    use crate::schema::users::dsl as users;

    let token = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let invitation_data: (
        u64,
        u64,
        Option<chrono::NaiveDate>,
        Option<chrono::NaiveDate>,
        u64,
        InvitationStatus,
        chrono::NaiveDateTime,
    ) = ri::renter_invitations
        .filter(ri::token.eq(&token))
        .select((
            ri::id,
            ri::apartment_id,
            ri::start_date,
            ri::end_date,
            ri::invited_by,
            ri::status,
            ri::expires_at,
        ))
        .first(&mut conn)
        .map_err(|_| AppError::NotFound)?;

    let (id, apartment_id, start_date, end_date, invited_by, status, expires_at) = invitation_data;

    if status != InvitationStatus::Pending {
        return Err(AppError::BadRequest(
            "This invitation has already been used or cancelled".into(),
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    if expires_at < now {
        diesel::update(ri::renter_invitations.filter(ri::id.eq(id)))
            .set(ri::status.eq(InvitationStatus::Expired))
            .execute(&mut conn)?;
        return Err(AppError::BadRequest("This invitation has expired".into()));
    }

    let (apt_number, building_id): (String, u64) = apt::apartments
        .filter(apt::id.eq(apartment_id))
        .select((apt::number, apt::building_id))
        .first(&mut conn)?;

    let building_address: String = bld::buildings
        .filter(bld::id.eq(building_id))
        .select(bld::address)
        .first(&mut conn)?;

    let invited_by_name: String = users::users
        .filter(users::id.eq(invited_by))
        .select(users::name)
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(InvitationInfo {
        id,
        apartment_number: apt_number,
        building_address,
        start_date,
        end_date,
        invited_by_name,
        expires_at,
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/invitations/{token}/accept",
    params(
        ("token" = String, Path, description = "Invitation token")
    ),
    responses(
        (status = 200, description = "Invitation accepted", body = AcceptInvitationResponse),
        (status = 401, description = "Unauthorized - must be logged in"),
        (status = 403, description = "Forbidden - email doesn't match invitation"),
        (status = 404, description = "Invitation not found"),
        (status = 410, description = "Invitation expired or already used")
    ),
    tag = "Invitations",
    security(("bearer_auth" = []))
)]
pub async fn accept_invitation(
    auth: AuthContext,
    path: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartment_renters::dsl as ar;
    use crate::schema::apartments::dsl as apt;
    use crate::schema::buildings::dsl as bld;
    use crate::schema::renter_invitations::dsl as ri;

    let token = path.into_inner();
    let user_id: u64 = auth
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::Internal("invalid_user_id".into()))?;
    let user_email = auth.claims.email.to_lowercase();

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let invitation_data: (
        u64,
        u64,
        String,
        Option<chrono::NaiveDate>,
        Option<chrono::NaiveDate>,
        InvitationStatus,
        chrono::NaiveDateTime,
    ) = ri::renter_invitations
        .filter(ri::token.eq(&token))
        .select((
            ri::id,
            ri::apartment_id,
            ri::email,
            ri::start_date,
            ri::end_date,
            ri::status,
            ri::expires_at,
        ))
        .first(&mut conn)
        .map_err(|_| AppError::NotFound)?;

    let (id, apartment_id, email, start_date, end_date, status, expires_at) = invitation_data;

    if email.to_lowercase() != user_email {
        return Err(AppError::Forbidden);
    }

    if status != InvitationStatus::Pending {
        return Err(AppError::BadRequest(
            "This invitation has already been used or cancelled".into(),
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    if expires_at < now {
        diesel::update(ri::renter_invitations.filter(ri::id.eq(id)))
            .set(ri::status.eq(InvitationStatus::Expired))
            .execute(&mut conn)?;
        return Err(AppError::BadRequest("This invitation has expired".into()));
    }

    let already_renter: Result<u64, _> = ar::apartment_renters
        .filter(
            ar::apartment_id
                .eq(apartment_id)
                .and(ar::user_id.eq(user_id)),
        )
        .select(ar::id)
        .first(&mut conn);

    if already_renter.is_ok() {
        diesel::update(ri::renter_invitations.filter(ri::id.eq(id)))
            .set((
                ri::status.eq(InvitationStatus::Accepted),
                ri::accepted_at.eq(Some(now)),
            ))
            .execute(&mut conn)?;

        return Err(AppError::BadRequest(
            "You are already a renter of this apartment".into(),
        ));
    }

    let new_renter = NewApartmentRenter {
        apartment_id,
        user_id,
        start_date,
        end_date,
        is_active: Some(true),
    };

    diesel::insert_into(ar::apartment_renters)
        .values(&new_renter)
        .execute(&mut conn)?;

    ensure_user_has_role(user_id, "Renter", &mut conn).await?;

    diesel::update(ri::renter_invitations.filter(ri::id.eq(id)))
        .set((
            ri::status.eq(InvitationStatus::Accepted),
            ri::accepted_at.eq(Some(now)),
        ))
        .execute(&mut conn)?;

    let (apt_number, building_id): (String, u64) = apt::apartments
        .filter(apt::id.eq(apartment_id))
        .select((apt::number, apt::building_id))
        .first(&mut conn)?;

    let building_address: String = bld::buildings
        .filter(bld::id.eq(building_id))
        .select(bld::address)
        .first(&mut conn)?;

    Ok(HttpResponse::Ok().json(AcceptInvitationResponse {
        success: true,
        message: "You have been successfully added as a renter".to_string(),
        apartment_id,
        apartment_number: apt_number,
        building_address,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/invitations/my",
    responses(
        (status = 200, description = "List of pending invitations for current user", body = Vec<RenterInvitationWithDetails>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Invitations",
    security(("bearer_auth" = []))
)]
pub async fn list_my_invitations(
    auth: AuthContext,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AppError> {
    use crate::schema::apartments::dsl as apt;
    use crate::schema::buildings::dsl as bld;
    use crate::schema::renter_invitations::dsl as ri;
    use crate::schema::users::dsl as users;

    let user_email = auth.claims.email.to_lowercase();

    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    let invitations_data: Vec<InvitationRow> = ri::renter_invitations
        .filter(
            ri::email
                .eq(&user_email)
                .and(ri::status.eq(InvitationStatus::Pending)),
        )
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

    let now = chrono::Utc::now().naive_utc();
    let mut result: Vec<RenterInvitationWithDetails> = Vec::new();

    for (
        id,
        apartment_id,
        email,
        start_date,
        end_date,
        invited_by,
        status,
        expires_at,
        created_at,
    ) in invitations_data
    {
        if expires_at < now {
            diesel::update(ri::renter_invitations.filter(ri::id.eq(id)))
                .set(ri::status.eq(InvitationStatus::Expired))
                .execute(&mut conn)?;
            continue;
        }

        let (apt_number, building_id): (String, u64) = apt::apartments
            .filter(apt::id.eq(apartment_id))
            .select((apt::number, apt::building_id))
            .first(&mut conn)?;

        let building_address: String = bld::buildings
            .filter(bld::id.eq(building_id))
            .select(bld::address)
            .first(&mut conn)?;

        let invited_by_name: String = users::users
            .filter(users::id.eq(invited_by))
            .select(users::name)
            .first(&mut conn)?;

        result.push(RenterInvitationWithDetails {
            id,
            apartment_id,
            apartment_number: apt_number,
            building_address,
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/invitations/my", web::get().to(list_my_invitations))
        .route("/invitations/{token}", web::get().to(get_invitation))
        .route(
            "/invitations/{token}/accept",
            web::post().to(accept_invitation),
        );
}
