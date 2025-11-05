use crate::auth::{AppError, AuthContext};
use crate::db::DbPool;
use crate::models::{NewUser, PublicUser, User}; // added PublicUser import
use crate::schema::{roles as roles_schema, user_roles as ur_schema};
use actix_web::{HttpResponse, Responder, web};
use diesel::prelude::*;

pub async fn list_users(pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    use crate::schema::users::dsl::*;
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    let user_list = users.select(User::as_select()).load(&mut conn)?;
    Ok(HttpResponse::Ok().json(user_list))
}

pub async fn create_user(
    auth: AuthContext,
    pool: web::Data<DbPool>,
    item: web::Json<NewUser>,
) -> Result<impl Responder, AppError> {
    use crate::schema::users::dsl as users_dsl;
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;
    diesel::insert_into(users_dsl::users)
        .values(&*item)
        .execute(&mut conn)?;
    Ok(HttpResponse::Created().finish())
}

#[derive(serde::Deserialize)]
pub struct SetRolesRequest {
    pub roles: Vec<String>,
}
#[derive(serde::Serialize)]
pub struct UserRolesResponse {
    pub user_id: u64,
    pub roles: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct UserWithRoles {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
}

pub async fn list_users_with_roles(auth: AuthContext, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin"]) { return Err(AppError::Forbidden); }
    use crate::schema::users::dsl as u;
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let all: Vec<User> = u::users.select(User::as_select()).load(&mut conn)?;
    let mut out = Vec::with_capacity(all.len());
    for usr in all.into_iter() {
        let roles = crate::auth::roles::get_user_roles(usr.id, &mut conn);
        out.push(UserWithRoles { id: usr.id, email: usr.email, name: usr.name, roles });
    }
    Ok(HttpResponse::Ok().json(out))
}

pub async fn set_user_roles(
    auth: AuthContext,
    path: web::Path<u64>,
    pool: web::Data<DbPool>,
    payload: web::Json<SetRolesRequest>,
) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin"]) {
        return Err(AppError::Forbidden);
    }
    let allowed: std::collections::HashSet<&'static str> =
        ["Admin", "Manager", "Homeowner", "Renter", "HOAMember"]
            .into_iter()
            .collect();
    for r in &payload.roles {
        if !allowed.contains(r.as_str()) {
            return Err(AppError::BadRequest(format!("invalid_role:{}", r)));
        }
    }
    let user_id = path.into_inner();
    let mut conn = pool
        .get()
        .map_err(|_| AppError::Internal("db_pool".into()))?;

    for role_name in &payload.roles {
        let role_id_res: Result<u64, _> = roles_schema::dsl::roles
            .filter(roles_schema::dsl::name.eq(role_name))
            .select(roles_schema::dsl::id)
            .first(&mut conn);
        let role_id_val = match role_id_res {
            Ok(idv) => idv,
            Err(_) => {
                diesel::insert_into(roles_schema::dsl::roles)
                    .values(roles_schema::dsl::name.eq(role_name))
                    .execute(&mut conn)?;
                roles_schema::dsl::roles
                    .filter(roles_schema::dsl::name.eq(role_name))
                    .select(roles_schema::dsl::id)
                    .first(&mut conn)?
            }
        };
        let exists: Result<(u64, u64), _> = ur_schema::dsl::user_roles
            .filter(
                ur_schema::dsl::user_id
                    .eq(user_id)
                    .and(ur_schema::dsl::role_id.eq(role_id_val)),
            )
            .select((ur_schema::dsl::user_id, ur_schema::dsl::role_id))
            .first(&mut conn);
        if exists.is_err() {
            diesel::insert_into(ur_schema::dsl::user_roles)
                .values((
                    ur_schema::dsl::user_id.eq(user_id),
                    ur_schema::dsl::role_id.eq(role_id_val),
                ))
                .execute(&mut conn)?;
        }
    }
    // remove roles no longer present in payload
    use crate::schema::user_roles::dsl as ur;
    use crate::schema::roles::dsl as rl;
    let current: Vec<(u64, String)> = ur::user_roles
        .inner_join(rl::roles.on(rl::id.eq(ur::role_id)))
        .filter(ur::user_id.eq(user_id))
        .select((rl::id, rl::name))
        .load(&mut conn)?;
    let desired: std::collections::HashSet<&str> = payload.roles.iter().map(|s| s.as_str()).collect();
    for (rid, rname) in current {
        if !desired.contains(rname.as_str()) {
            diesel::delete(ur::user_roles.filter(ur::user_id.eq(user_id).and(ur::role_id.eq(rid)))).execute(&mut conn)?;
        }
    }
    let assigned: Vec<String> = ur_schema::dsl::user_roles
        .inner_join(roles_schema::dsl::roles.on(roles_schema::dsl::id.eq(ur_schema::dsl::role_id)))
        .filter(ur_schema::dsl::user_id.eq(user_id))
        .select(roles_schema::dsl::name)
        .load(&mut conn)?;
    Ok(HttpResponse::Ok().json(UserRolesResponse {
        user_id,
        roles: assigned,
    }))
}

pub async fn list_public_users(auth: AuthContext, pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    if !auth.has_any_role(&["Admin", "Manager"]) { return Err(AppError::Forbidden); }
    use crate::schema::users::dsl as u;
    let mut conn = pool.get().map_err(|_| AppError::Internal("db_pool".into()))?;
    let raw: Vec<User> = u::users.select(User::as_select()).load(&mut conn)?;
    let list: Vec<PublicUser> = raw.into_iter().map(PublicUser::from).collect();
    Ok(HttpResponse::Ok().json(list))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/users", web::get().to(list_users))
        .route("/users", web::post().to(create_user))
        .route("/users/public", web::get().to(list_public_users))
        .route("/users/with_roles", web::get().to(list_users_with_roles))
        .route("/users/{id}/roles", web::post().to(set_user_roles));
}
