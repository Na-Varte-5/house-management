CREATE TABLE buildings (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    address VARCHAR(255) NOT NULL,
    construction_year INT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE apartments (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    building_id BIGINT UNSIGNED NOT NULL,
    number VARCHAR(64) NOT NULL,
    size_sq_m DOUBLE NULL,
    bedrooms INT NULL,
    bathrooms INT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_apartments_building FOREIGN KEY (building_id) REFERENCES buildings(id) ON DELETE CASCADE
);
