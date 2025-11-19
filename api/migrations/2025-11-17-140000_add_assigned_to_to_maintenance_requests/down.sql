-- Revert assigned_to column addition
ALTER TABLE maintenance_requests DROP FOREIGN KEY fk_mr_assigned;
ALTER TABLE maintenance_requests DROP COLUMN assigned_to;
DROP INDEX idx_mr_assigned_to ON maintenance_requests;
