-- Migration: create voting related tables
CREATE TABLE proposals (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  title VARCHAR(255) NOT NULL,
  description TEXT NOT NULL,
  created_by BIGINT UNSIGNED NOT NULL,
  start_time DATETIME NOT NULL,
  end_time DATETIME NOT NULL,
  voting_method VARCHAR(32) NOT NULL,
  eligible_roles VARCHAR(255) NOT NULL,
  status VARCHAR(16) NOT NULL,
  created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_proposal_creator FOREIGN KEY (created_by) REFERENCES users(id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE votes (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  proposal_id BIGINT UNSIGNED NOT NULL,
  user_id BIGINT UNSIGNED NOT NULL,
  weight_decimal DECIMAL(18,6) NOT NULL,
  choice VARCHAR(16) NOT NULL,
  created_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE KEY uq_vote_once (proposal_id, user_id),
  CONSTRAINT fk_vote_proposal FOREIGN KEY (proposal_id) REFERENCES proposals(id),
  CONSTRAINT fk_vote_user FOREIGN KEY (user_id) REFERENCES users(id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE proposal_results (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  proposal_id BIGINT UNSIGNED NOT NULL UNIQUE,
  passed BOOLEAN NOT NULL,
  yes_weight DECIMAL(18,6) NOT NULL,
  no_weight DECIMAL(18,6) NOT NULL,
  abstain_weight DECIMAL(18,6) NOT NULL,
  total_weight DECIMAL(18,6) NOT NULL,
  tallied_at TIMESTAMP NULL DEFAULT CURRENT_TIMESTAMP,
  method_applied_version VARCHAR(16) NOT NULL,
  CONSTRAINT fk_result_proposal FOREIGN KEY (proposal_id) REFERENCES proposals(id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_proposals_status ON proposals(status);
CREATE INDEX idx_proposals_start_end ON proposals(start_time, end_time);
CREATE INDEX idx_votes_proposal ON votes(proposal_id);
