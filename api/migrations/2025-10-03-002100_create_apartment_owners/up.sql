CREATE TABLE apartment_owners (
    apartment_id BIGINT UNSIGNED NOT NULL,
    user_id BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (apartment_id, user_id),
    CONSTRAINT fk_apartment_owners_apartment FOREIGN KEY (apartment_id) REFERENCES apartments(id) ON DELETE CASCADE,
    CONSTRAINT fk_apartment_owners_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
