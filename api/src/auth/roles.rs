use crate::schema::{roles, user_roles, users};
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;

pub fn get_user_roles(user_id: u64, conn: &mut MysqlConnection) -> Vec<String> {
    use roles::dsl as r;
    use user_roles::dsl as ur;
    ur::user_roles
        .inner_join(r::roles.on(r::id.eq(ur::role_id)))
        .filter(ur::user_id.eq(user_id))
        .select(r::name)
        .load::<String>(conn)
        .unwrap_or_default()
}
pub fn count_users(conn: &mut MysqlConnection) -> i64 {
    users::table
        .count()
        .get_result::<i64>(conn)
        .unwrap_or_default()
}
pub fn ensure_role(name: &str, conn: &mut MysqlConnection) -> Result<u64, diesel::result::Error> {
    use roles::dsl as r;
    if let Ok(found) = r::roles
        .filter(r::name.eq(name))
        .select(r::id)
        .first::<u64>(conn)
    {
        return Ok(found);
    }
    diesel::insert_into(r::roles)
        .values((r::name.eq(name),))
        .execute(conn)?;
    r::roles
        .filter(r::name.eq(name))
        .select(r::id)
        .first::<u64>(conn)
}
pub fn assign_role(
    user_id_v: u64,
    role_id_v: u64,
    conn: &mut MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use user_roles::dsl as ur;
    let exists: Result<(u64, u64), _> = ur::user_roles
        .filter(ur::user_id.eq(user_id_v).and(ur::role_id.eq(role_id_v)))
        .select((ur::user_id, ur::role_id))
        .first(conn);
    if exists.is_ok() {
        return Ok(());
    }
    diesel::insert_into(ur::user_roles)
        .values((ur::user_id.eq(user_id_v), ur::role_id.eq(role_id_v)))
        .execute(conn)?;
    Ok(())
}
pub fn has_any_role(claims_roles: &[String], wanted: &[&str]) -> bool {
    if wanted.is_empty() {
        return true;
    }
    claims_roles.iter().any(|r| wanted.iter().any(|w| r == w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_any_role_positive() {
        let claims_roles = vec!["Admin".to_string(), "Homeowner".to_string()];
        assert!(has_any_role(&claims_roles, &["Admin"]));
        assert!(has_any_role(&claims_roles, &["Homeowner"]));
        assert!(has_any_role(&claims_roles, &["Manager", "Admin"]));
    }

    #[test]
    fn test_has_any_role_negative() {
        let claims_roles = vec!["Renter".to_string()];
        assert!(!has_any_role(&claims_roles, &["Admin"]));
        assert!(!has_any_role(&claims_roles, &["Manager", "Admin"]));
    }

    #[test]
    fn test_has_any_role_empty_wanted() {
        let claims_roles = vec!["Renter".to_string()];
        assert!(has_any_role(&claims_roles, &[])); // empty means allow
    }
}
