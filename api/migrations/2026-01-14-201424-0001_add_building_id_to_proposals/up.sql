ALTER TABLE proposals
ADD COLUMN building_id BIGINT UNSIGNED NULL AFTER created_by,
ADD CONSTRAINT fk_proposals_building FOREIGN KEY (building_id) REFERENCES buildings(id) ON DELETE SET NULL;

CREATE INDEX idx_proposals_building ON proposals(building_id);
