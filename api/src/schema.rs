// @generated automatically by Diesel CLI.

diesel::table! {
    announcements (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        title -> Varchar,
        body_md -> Text,
        body_html -> Text,
        author_id -> Unsigned<Bigint>,
        public -> Bool,
        pinned -> Bool,
        roles_csv -> Nullable<Text>,
        building_id -> Nullable<Unsigned<Bigint>>,
        apartment_id -> Nullable<Unsigned<Bigint>>,
        comments_enabled -> Bool,
        publish_at -> Nullable<Timestamp>,
        expire_at -> Nullable<Timestamp>,
        is_deleted -> Bool,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    announcements_comments (id) {
        id -> Unsigned<Bigint>,
        announcement_id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        body_md -> Text,
        body_html -> Text,
        is_deleted -> Bool,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    apartment_owners (apartment_id, user_id) {
        apartment_id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
    }
}

diesel::table! {
    apartment_renters (id) {
        id -> Unsigned<Bigint>,
        apartment_id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        start_date -> Nullable<Date>,
        end_date -> Nullable<Date>,
        is_active -> Nullable<Bool>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    apartments (id) {
        id -> Unsigned<Bigint>,
        building_id -> Unsigned<Bigint>,
        #[max_length = 64]
        number -> Varchar,
        size_sq_m -> Nullable<Double>,
        bedrooms -> Nullable<Integer>,
        bathrooms -> Nullable<Integer>,
        created_at -> Nullable<Timestamp>,
        is_deleted -> Bool,
    }
}

diesel::table! {
    building_managers (building_id, user_id) {
        building_id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    buildings (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        address -> Varchar,
        construction_year -> Nullable<Integer>,
        created_at -> Nullable<Timestamp>,
        is_deleted -> Bool,
    }
}

diesel::table! {
    maintenance_request_attachments (id) {
        id -> Unsigned<Bigint>,
        request_id -> Unsigned<Bigint>,
        #[max_length = 255]
        original_filename -> Varchar,
        #[max_length = 255]
        stored_filename -> Varchar,
        #[max_length = 128]
        mime_type -> Varchar,
        size_bytes -> Unsigned<Bigint>,
        is_deleted -> Bool,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    maintenance_request_history (id) {
        id -> Unsigned<Bigint>,
        request_id -> Unsigned<Bigint>,
        #[max_length = 32]
        from_status -> Nullable<Varchar>,
        #[max_length = 32]
        to_status -> Varchar,
        note -> Nullable<Text>,
        changed_by -> Unsigned<Bigint>,
        changed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    maintenance_requests (id) {
        id -> Unsigned<Bigint>,
        apartment_id -> Unsigned<Bigint>,
        created_by -> Unsigned<Bigint>,
        assigned_to -> Nullable<Unsigned<Bigint>>,
        #[max_length = 32]
        request_type -> Varchar,
        #[max_length = 16]
        priority -> Varchar,
        #[max_length = 255]
        title -> Varchar,
        description -> Text,
        #[max_length = 32]
        status -> Varchar,
        resolution_notes -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    meter_readings (id) {
        id -> Unsigned<Bigint>,
        meter_id -> Unsigned<Bigint>,
        reading_value -> Decimal,
        reading_timestamp -> Datetime,
        #[max_length = 16]
        unit -> Varchar,
        #[max_length = 32]
        source -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    meters (id) {
        id -> Unsigned<Bigint>,
        apartment_id -> Unsigned<Bigint>,
        #[max_length = 32]
        meter_type -> Varchar,
        #[max_length = 128]
        serial_number -> Varchar,
        is_visible_to_renters -> Bool,
        installation_date -> Nullable<Date>,
        calibration_due_date -> Nullable<Date>,
        last_calibration_date -> Nullable<Date>,
        is_active -> Bool,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    proposal_results (id) {
        id -> Unsigned<Bigint>,
        proposal_id -> Unsigned<Bigint>,
        passed -> Bool,
        yes_weight -> Decimal,
        no_weight -> Decimal,
        abstain_weight -> Decimal,
        total_weight -> Decimal,
        tallied_at -> Nullable<Timestamp>,
        #[max_length = 16]
        method_applied_version -> Varchar,
    }
}

diesel::table! {
    proposals (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        title -> Varchar,
        description -> Text,
        created_by -> Unsigned<Bigint>,
        building_id -> Nullable<Unsigned<Bigint>>,
        start_time -> Datetime,
        end_time -> Datetime,
        #[max_length = 32]
        voting_method -> Varchar,
        #[max_length = 255]
        eligible_roles -> Varchar,
        #[max_length = 16]
        status -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    roles (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 64]
        name -> Varchar,
    }
}

diesel::table! {
    user_roles (user_id, role_id) {
        user_id -> Unsigned<Bigint>,
        role_id -> Unsigned<Bigint>,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    votes (id) {
        id -> Unsigned<Bigint>,
        proposal_id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        weight_decimal -> Decimal,
        #[max_length = 16]
        choice -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    webhook_api_keys (id) {
        id -> Unsigned<Bigint>,
        #[max_length = 128]
        name -> Varchar,
        #[max_length = 255]
        api_key_hash -> Varchar,
        is_active -> Bool,
        created_by -> Unsigned<Bigint>,
        created_at -> Nullable<Timestamp>,
        last_used_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(announcements -> apartments (apartment_id));
diesel::joinable!(announcements -> buildings (building_id));
diesel::joinable!(announcements -> users (author_id));
diesel::joinable!(announcements_comments -> announcements (announcement_id));
diesel::joinable!(announcements_comments -> users (user_id));
diesel::joinable!(apartment_owners -> apartments (apartment_id));
diesel::joinable!(apartment_owners -> users (user_id));
diesel::joinable!(apartments -> buildings (building_id));
diesel::joinable!(maintenance_request_attachments -> maintenance_requests (request_id));
diesel::joinable!(maintenance_request_history -> maintenance_requests (request_id));
diesel::joinable!(maintenance_request_history -> users (changed_by));
diesel::joinable!(maintenance_requests -> apartments (apartment_id));
diesel::joinable!(meter_readings -> meters (meter_id));
diesel::joinable!(meters -> apartments (apartment_id));
diesel::joinable!(proposal_results -> proposals (proposal_id));
diesel::joinable!(proposals -> buildings (building_id));
diesel::joinable!(proposals -> users (created_by));
diesel::joinable!(votes -> proposals (proposal_id));
diesel::joinable!(votes -> users (user_id));
diesel::joinable!(webhook_api_keys -> users (created_by));

diesel::allow_tables_to_appear_in_same_query!(
    announcements,
    announcements_comments,
    apartment_owners,
    apartment_renters,
    apartments,
    building_managers,
    buildings,
    maintenance_request_attachments,
    maintenance_request_history,
    maintenance_requests,
    meter_readings,
    meters,
    proposal_results,
    proposals,
    roles,
    user_roles,
    users,
    votes,
    webhook_api_keys,
);
