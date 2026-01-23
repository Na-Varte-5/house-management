use crate::auth::error::AppError;
use diesel::prelude::*;

/// Returns Option<Vec<u64>> of building IDs the user can access.
/// Returns None for Admin (no filter needed - sees all buildings).
/// Returns Some(Vec) for other users with accessible buildings.
pub fn get_user_building_ids(
    user_id: u64,
    is_admin: bool,
    conn: &mut MysqlConnection,
) -> Result<Option<Vec<u64>>, AppError> {
    // Admin sees all - return None to indicate "no filter"
    if is_admin {
        return Ok(None);
    }

    use crate::schema::{
        apartment_owners::dsl as ao, apartment_renters::dsl as ar, apartments::dsl as apt,
        building_managers::dsl as bm,
    };

    // Get buildings from owned apartments
    let owned_buildings: Vec<u64> = ao::apartment_owners
        .inner_join(apt::apartments.on(apt::id.eq(ao::apartment_id)))
        .filter(ao::user_id.eq(user_id))
        .filter(apt::is_deleted.eq(false))
        .select(apt::building_id)
        .distinct()
        .load(conn)?;

    // Get buildings from rented apartments (where is_active=true)
    let rented_buildings: Vec<u64> = ar::apartment_renters
        .inner_join(apt::apartments.on(apt::id.eq(ar::apartment_id)))
        .filter(ar::user_id.eq(user_id))
        .filter(ar::is_active.eq(true))
        .filter(apt::is_deleted.eq(false))
        .select(apt::building_id)
        .distinct()
        .load(conn)?;

    // Get directly managed buildings
    let managed_buildings: Vec<u64> = bm::building_managers
        .filter(bm::user_id.eq(user_id))
        .select(bm::building_id)
        .load(conn)?;

    // Combine and deduplicate
    let mut all_buildings: Vec<u64> = owned_buildings;
    all_buildings.extend(rented_buildings);
    all_buildings.extend(managed_buildings);
    all_buildings.sort_unstable();
    all_buildings.dedup();

    Ok(Some(all_buildings))
}
