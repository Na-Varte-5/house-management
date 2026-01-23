use crate::auth::AppError;
use diesel::prelude::*;

/// Check if a user owns a specific apartment
pub(super) fn user_owns_apartment(
    user_id: u64,
    apartment_id: u64,
    conn: &mut diesel::MysqlConnection,
) -> Result<bool, AppError> {
    use crate::schema::apartment_owners::dsl as ao;

    let count: i64 = ao::apartment_owners
        .filter(ao::apartment_id.eq(apartment_id))
        .filter(ao::user_id.eq(user_id))
        .count()
        .get_result(conn)?;

    Ok(count > 0)
}
