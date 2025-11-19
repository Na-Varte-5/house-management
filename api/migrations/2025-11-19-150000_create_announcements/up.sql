-- Migration: create announcements and comments tables
CREATE TABLE announcements (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  title VARCHAR(255) NOT NULL,
  body_md TEXT NOT NULL,
  body_html TEXT NOT NULL,
  author_id BIGINT UNSIGNED NOT NULL,
  public BOOLEAN NOT NULL DEFAULT FALSE,
  pinned BOOLEAN NOT NULL DEFAULT FALSE,
  roles_csv TEXT NULL,
  building_id BIGINT UNSIGNED NULL,
  apartment_id BIGINT UNSIGNED NULL,
  comments_enabled BOOLEAN NOT NULL DEFAULT FALSE,
  publish_at TIMESTAMP NULL,
  expire_at TIMESTAMP NULL,
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_announcement_author FOREIGN KEY (author_id) REFERENCES users(id),
  CONSTRAINT fk_announcement_building FOREIGN KEY (building_id) REFERENCES buildings(id) ON DELETE SET NULL,
  CONSTRAINT fk_announcement_apartment FOREIGN KEY (apartment_id) REFERENCES apartments(id) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE announcements_comments (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  announcement_id BIGINT UNSIGNED NOT NULL,
  user_id BIGINT UNSIGNED NOT NULL,
  body_md TEXT NOT NULL,
  body_html TEXT NOT NULL,
  is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_comment_announcement FOREIGN KEY (announcement_id) REFERENCES announcements(id) ON DELETE CASCADE,
  CONSTRAINT fk_comment_user FOREIGN KEY (user_id) REFERENCES users(id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_announcements_pub ON announcements(public, pinned, publish_at, expire_at, is_deleted);
CREATE INDEX idx_announcements_building ON announcements(building_id);
CREATE INDEX idx_announcements_apartment ON announcements(apartment_id);
CREATE INDEX idx_comments_announcement ON announcements_comments(announcement_id, is_deleted, created_at);
