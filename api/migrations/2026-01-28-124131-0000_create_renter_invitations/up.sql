CREATE TABLE renter_invitations (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    apartment_id BIGINT UNSIGNED NOT NULL,
    email VARCHAR(255) NOT NULL,
    token VARCHAR(128) NOT NULL UNIQUE,
    start_date DATE,
    end_date DATE,
    invited_by BIGINT UNSIGNED NOT NULL,
    status ENUM('pending', 'accepted', 'expired', 'cancelled') NOT NULL DEFAULT 'pending',
    expires_at DATETIME NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    accepted_at TIMESTAMP NULL,
    FOREIGN KEY (apartment_id) REFERENCES apartments(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_renter_invitations_token (token),
    INDEX idx_renter_invitations_email (email),
    INDEX idx_renter_invitations_apartment (apartment_id),
    INDEX idx_renter_invitations_status (status)
);
