-- Drop performance indexes

DROP INDEX IF EXISTS idx_user_roles_role ON user_roles;
DROP INDEX IF EXISTS idx_user_roles_user ON user_roles;
DROP INDEX IF EXISTS idx_maintenance_attachments_request ON maintenance_request_attachments;
DROP INDEX IF EXISTS idx_maintenance_history_request ON maintenance_request_history;
DROP INDEX IF EXISTS idx_apartment_owners_user ON apartment_owners;
DROP INDEX IF EXISTS idx_apartment_owners_apartment ON apartment_owners;
DROP INDEX IF EXISTS idx_apartments_building ON apartments;
DROP INDEX IF EXISTS idx_maintenance_apartment ON maintenance_requests;
DROP INDEX IF EXISTS idx_maintenance_assigned_to ON maintenance_requests;
DROP INDEX IF EXISTS idx_maintenance_created_by ON maintenance_requests;
DROP INDEX IF EXISTS idx_maintenance_status ON maintenance_requests;
DROP INDEX IF EXISTS idx_maintenance_attachments_deleted ON maintenance_request_attachments;
DROP INDEX IF EXISTS idx_apartments_deleted ON apartments;
DROP INDEX IF EXISTS idx_buildings_deleted ON buildings;
