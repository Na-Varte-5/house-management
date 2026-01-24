CREATE TABLE property_history (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    apartment_id BIGINT UNSIGNED NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    user_id BIGINT UNSIGNED,
    changed_by BIGINT UNSIGNED NOT NULL,
    description TEXT NOT NULL,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (apartment_id) REFERENCES apartments(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (changed_by) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_apartment_id (apartment_id),
    INDEX idx_created_at (created_at)
);
