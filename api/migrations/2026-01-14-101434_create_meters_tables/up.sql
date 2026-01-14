-- Migration: create meters and meter_readings tables with webhook API keys
CREATE TABLE meters (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  apartment_id BIGINT UNSIGNED NOT NULL,
  meter_type VARCHAR(32) NOT NULL,
  serial_number VARCHAR(128) NOT NULL,
  is_visible_to_renters BOOLEAN NOT NULL DEFAULT TRUE,
  installation_date DATE NULL,
  calibration_due_date DATE NULL,
  last_calibration_date DATE NULL,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_meter_apartment FOREIGN KEY (apartment_id) REFERENCES apartments(id),
  UNIQUE KEY uk_serial_number (serial_number)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE meter_readings (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  meter_id BIGINT UNSIGNED NOT NULL,
  reading_value DECIMAL(15,4) NOT NULL,
  reading_timestamp DATETIME NOT NULL,
  unit VARCHAR(16) NOT NULL,
  source VARCHAR(32) NOT NULL,
  created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_reading_meter FOREIGN KEY (meter_id) REFERENCES meters(id),
  UNIQUE KEY uk_meter_timestamp (meter_id, reading_timestamp)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE webhook_api_keys (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  name VARCHAR(128) NOT NULL,
  api_key_hash VARCHAR(255) NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  created_by BIGINT UNSIGNED NOT NULL,
  created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
  last_used_at TIMESTAMP NULL,
  CONSTRAINT fk_apikey_user FOREIGN KEY (created_by) REFERENCES users(id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Indexes for performance
CREATE INDEX idx_meter_apartment ON meters(apartment_id);
CREATE INDEX idx_meter_calibration_due ON meters(calibration_due_date);
CREATE INDEX idx_meter_type ON meters(meter_type);
CREATE INDEX idx_meter_active ON meters(is_active);
CREATE INDEX idx_reading_timestamp ON meter_readings(reading_timestamp);
CREATE INDEX idx_reading_meter ON meter_readings(meter_id);
CREATE INDEX idx_apikey_active ON webhook_api_keys(is_active);
