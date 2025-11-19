-- Migration: add assigned_to column to maintenance_requests
ALTER TABLE maintenance_requests ADD COLUMN assigned_to BIGINT UNSIGNED NULL AFTER created_by;
ALTER TABLE maintenance_requests ADD CONSTRAINT fk_mr_assigned FOREIGN KEY (assigned_to) REFERENCES users(id);
CREATE INDEX idx_mr_assigned_to ON maintenance_requests(assigned_to);
