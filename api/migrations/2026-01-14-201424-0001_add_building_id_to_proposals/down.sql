ALTER TABLE proposals DROP FOREIGN KEY fk_proposals_building;
ALTER TABLE proposals DROP INDEX idx_proposals_building;
ALTER TABLE proposals DROP COLUMN building_id;
