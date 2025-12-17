-- Add indexes for performance on frequently filtered columns

-- Soft-delete indexes for faster active/deleted queries
CREATE INDEX idx_buildings_deleted ON buildings(is_deleted);
CREATE INDEX idx_apartments_deleted ON apartments(is_deleted);
CREATE INDEX idx_maintenance_attachments_deleted ON maintenance_request_attachments(is_deleted);

-- Maintenance request indexes for common queries
CREATE INDEX idx_maintenance_status ON maintenance_requests(status, created_at);
CREATE INDEX idx_maintenance_created_by ON maintenance_requests(created_by);
CREATE INDEX idx_maintenance_assigned_to ON maintenance_requests(assigned_to);
CREATE INDEX idx_maintenance_apartment ON maintenance_requests(apartment_id);

-- Foreign key indexes for joins
CREATE INDEX idx_apartments_building ON apartments(building_id);
CREATE INDEX idx_apartment_owners_apartment ON apartment_owners(apartment_id);
CREATE INDEX idx_apartment_owners_user ON apartment_owners(user_id);
CREATE INDEX idx_maintenance_history_request ON maintenance_request_history(request_id);
CREATE INDEX idx_maintenance_attachments_request ON maintenance_request_attachments(request_id);

-- User roles for RBAC checks
CREATE INDEX idx_user_roles_user ON user_roles(user_id);
CREATE INDEX idx_user_roles_role ON user_roles(role_id);
