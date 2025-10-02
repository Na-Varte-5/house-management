// @generated automatically by Diesel CLI.

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

diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    roles,
    user_roles,
    users,
);
