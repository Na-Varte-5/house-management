CREATE TABLE apartment_renters (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    apartment_id BIGINT UNSIGNED NOT NULL,
    user_id BIGINT UNSIGNED NOT NULL,
    start_date DATE,
    end_date DATE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (apartment_id) REFERENCES apartments(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY unique_active_renter (apartment_id, user_id, is_active)
);

CREATE INDEX idx_apartment_renters_user ON apartment_renters(user_id);
CREATE INDEX idx_apartment_renters_active ON apartment_renters(is_active);
