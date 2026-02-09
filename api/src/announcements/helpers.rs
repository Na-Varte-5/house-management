use super::types::AnnouncementOut;
use crate::auth::error::AppError;
use crate::db::DbPool;
use crate::models::Announcement;
use actix_web::web;
use diesel::prelude::*;
use std::collections::HashMap;

pub(super) fn render_markdown(md: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(md, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    // Ammonia's clean now returns a Document; convert it to String.
    ammonia::Builder::default().clean(&html_output).to_string()
}

pub(super) fn conn(
    pool: &web::Data<DbPool>,
) -> Result<
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::mysql::MysqlConnection>>,
    AppError,
> {
    pool.get().map_err(|_| AppError::Internal("db_pool".into()))
}

pub(super) fn enrich(
    mut anns: Vec<Announcement>,
    conn: &mut diesel::MysqlConnection,
) -> Result<Vec<AnnouncementOut>, AppError> {
    use crate::schema::apartments::dsl as ap;
    use crate::schema::buildings::dsl as b;
    use crate::schema::users::dsl as u;
    let ids: Vec<u64> = anns.iter().map(|a| a.author_id).collect();
    let users = u::users
        .filter(u::id.eq_any(&ids))
        .load::<crate::models::User>(conn)?;
    let mut user_map: HashMap<u64, String> = HashMap::new();
    for usr in users {
        user_map.insert(usr.id, usr.name);
    }
    let b_ids: Vec<u64> = anns
        .iter()
        .filter_map(|a| a.building_id)
        .collect::<Vec<_>>();
    let b_ids_uniq: Vec<u64> = {
        let mut tmp = b_ids.clone();
        tmp.sort_unstable();
        std::mem::take(&mut tmp)
    };
    let building_rows: Vec<(u64, String)> = if b_ids_uniq.is_empty() {
        vec![]
    } else {
        b::buildings
            .filter(b::id.eq_any(&b_ids_uniq))
            .select((b::id, b::address))
            .load(conn)?
    };
    let mut building_map: HashMap<u64, String> = HashMap::new();
    for (bid, addr) in building_rows {
        building_map.insert(bid, addr);
    }
    let ap_ids: Vec<u64> = anns
        .iter()
        .filter_map(|a| a.apartment_id)
        .collect::<Vec<_>>();
    let ap_ids_uniq: Vec<u64> = {
        let mut tmp = ap_ids.clone();
        tmp.sort_unstable();
        std::mem::take(&mut tmp)
    };
    let apartment_rows: Vec<(u64, String)> = if ap_ids_uniq.is_empty() {
        vec![]
    } else {
        ap::apartments
            .filter(ap::id.eq_any(&ap_ids_uniq))
            .select((ap::id, ap::number))
            .load(conn)?
    };
    let mut apartment_map: HashMap<u64, String> = HashMap::new();
    for (aid, num) in apartment_rows {
        apartment_map.insert(aid, num);
    }
    Ok(anns
        .drain(..)
        .map(|a| AnnouncementOut {
            author_name: user_map
                .get(&a.author_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".into()),
            id: a.id,
            title: a.title,
            body_md: a.body_md,
            body_html: a.body_html,
            author_id: a.author_id,
            public: a.public,
            pinned: a.pinned,
            roles_csv: a.roles_csv,
            building_id: a.building_id,
            building_address: a
                .building_id
                .and_then(|bid| building_map.get(&bid).cloned()),
            apartment_id: a.apartment_id,
            apartment_number: a
                .apartment_id
                .and_then(|aid| apartment_map.get(&aid).cloned()),
            comments_enabled: a.comments_enabled,
            publish_at: a.publish_at,
            expire_at: a.expire_at,
            is_deleted: a.is_deleted,
            created_at: a.created_at,
            updated_at: a.updated_at,
        })
        .collect())
}

pub(super) fn enrich_one(
    a: Announcement,
    conn: &mut diesel::MysqlConnection,
) -> Result<AnnouncementOut, AppError> {
    use crate::schema::apartments::dsl as ap;
    use crate::schema::buildings::dsl as b;
    use crate::schema::users::dsl as u;
    let name = u::users
        .filter(u::id.eq(a.author_id))
        .select(u::name)
        .first::<String>(conn)
        .unwrap_or_else(|_| "Unknown".into());
    let building_address = match a.building_id {
        Some(bid) => b::buildings
            .filter(b::id.eq(bid))
            .select(b::address)
            .first::<String>(conn)
            .ok(),
        None => None,
    };
    let apartment_number = match a.apartment_id {
        Some(aid) => ap::apartments
            .filter(ap::id.eq(aid))
            .select(ap::number)
            .first::<String>(conn)
            .ok(),
        None => None,
    };
    Ok(AnnouncementOut {
        author_name: name,
        id: a.id,
        title: a.title,
        body_md: a.body_md,
        body_html: a.body_html,
        author_id: a.author_id,
        public: a.public,
        pinned: a.pinned,
        roles_csv: a.roles_csv,
        building_id: a.building_id,
        building_address,
        apartment_id: a.apartment_id,
        apartment_number,
        comments_enabled: a.comments_enabled,
        publish_at: a.publish_at,
        expire_at: a.expire_at,
        is_deleted: a.is_deleted,
        created_at: a.created_at,
        updated_at: a.updated_at,
    })
}
