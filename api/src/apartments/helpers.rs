use crate::auth::AppError;
use crate::models::NewPropertyHistory;
use diesel::prelude::*;

/// Helper function to ensure a user has a specific role.
/// Creates the role if it doesn't exist, then assigns it to the user if not already assigned.
pub(super) async fn ensure_user_has_role(
    user_id: u64,
    role_name: &str,
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::MysqlConnection>,
    >,
) -> Result<(), AppError> {
    use crate::schema::roles::dsl as roles_schema;
    use crate::schema::user_roles::dsl as ur_schema;

    // Get or create role
    let role_id_res: Result<u64, _> = roles_schema::roles
        .filter(roles_schema::name.eq(role_name))
        .select(roles_schema::id)
        .first(conn);

    let role_id = match role_id_res {
        Ok(id) => id,
        Err(_) => {
            // Create the role if it doesn't exist
            diesel::insert_into(roles_schema::roles)
                .values(roles_schema::name.eq(role_name))
                .execute(conn)?;
            roles_schema::roles
                .filter(roles_schema::name.eq(role_name))
                .select(roles_schema::id)
                .first(conn)?
        }
    };

    // Check if user already has this role
    let exists: Result<(u64, u64), _> = ur_schema::user_roles
        .filter(
            ur_schema::user_id
                .eq(user_id)
                .and(ur_schema::role_id.eq(role_id)),
        )
        .select((ur_schema::user_id, ur_schema::role_id))
        .first(conn);

    // Assign role if not already assigned
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

/// Helper function to remove a role from a user if they have no more property assignments.
/// Checks if the user owns any apartments or is an active renter.
/// If no assignments exist, removes the specified role.
pub(super) async fn remove_role_if_no_assignments(
    user_id: u64,
    role_name: &str,
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::MysqlConnection>,
    >,
) -> Result<(), AppError> {
    use crate::schema::apartment_owners::dsl as ao;
    use crate::schema::apartment_renters::dsl as ar;
    use crate::schema::roles::dsl as roles_schema;
    use crate::schema::user_roles::dsl as ur_schema;

    // Determine which tables to check based on role
    let has_assignments = match role_name {
        "Homeowner" => {
            // Check if user owns any apartments
            let count: i64 = ao::apartment_owners
                .filter(ao::user_id.eq(user_id))
                .count()
                .get_result(conn)?;
            count > 0
        }
        "Renter" => {
            // Check if user has any active rental assignments
            let count: i64 = ar::apartment_renters
                .filter(ar::user_id.eq(user_id).and(ar::is_active.eq(true)))
                .count()
                .get_result(conn)?;
            count > 0
        }
        _ => return Ok(()), // Don't auto-remove other roles
    };

    // If no assignments, remove the role
    if !has_assignments {
        let role_id_res: Result<u64, _> = roles_schema::roles
            .filter(roles_schema::name.eq(role_name))
            .select(roles_schema::id)
            .first(conn);

        if let Ok(role_id) = role_id_res {
            diesel::delete(
                ur_schema::user_roles.filter(
                    ur_schema::user_id
                        .eq(user_id)
                        .and(ur_schema::role_id.eq(role_id)),
                ),
            )
            .execute(conn)?;
        }
    }

    Ok(())
}

/// Helper function to log property history events
pub(super) async fn log_property_event(
    apartment_id: u64,
    event_type: &str,
    user_id: Option<u64>,
    changed_by: u64,
    description: String,
    metadata: Option<String>,
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::MysqlConnection>,
    >,
) -> Result<(), AppError> {
    use crate::schema::property_history::dsl as ph;

    let new_event = NewPropertyHistory {
        apartment_id,
        event_type: event_type.to_string(),
        user_id,
        changed_by,
        description,
        metadata,
    };

    diesel::insert_into(ph::property_history)
        .values(&new_event)
        .execute(conn)?;

    Ok(())
}
