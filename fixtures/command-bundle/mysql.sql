-- Explicit installation; same physical database/connection, both InnoDB.
-- No triggers, generated/default/auto_increment columns or external effects.
CREATE TABLE quent_dispatch_test.bundle_items (
    tenant BIGINT NOT NULL,
    id BIGINT NOT NULL,
    amount BIGINT NOT NULL CHECK (amount >= 0),
    flag BOOLEAN NOT NULL,
    note TEXT NULL,
    PRIMARY KEY (tenant, id)
) ENGINE=InnoDB;
CREATE TABLE quent_dispatch_test.bundle_ledger (
    identity VARBINARY(2048) PRIMARY KEY,
    input LONGBLOB NOT NULL,
    result VARBINARY(29) NULL,
    version SMALLINT NOT NULL,
    CHECK (OCTET_LENGTH(identity) BETWEEN 1 AND 2048),
    CHECK (OCTET_LENGTH(input) BETWEEN 4 AND 1048576),
    CHECK (result IS NULL OR OCTET_LENGTH(result) = CASE WHEN version = 1 THEN 13 ELSE 29 END),
    CHECK (version IN (1, 2))
) ENGINE=InnoDB;
-- Strict SQL mode and SQLx CLIENT_FOUND_ROWS required. Install outside command
-- transactions: MySQL DDL implicitly commits. No nontransactional engine support.
-- Application role: exact-table SELECT/INSERT/UPDATE; retention owner manages
-- deletion separately and must not delete identities while reuse is possible.

CREATE TABLE quent_dispatch_test.bundle_audit (
    tenant BIGINT NOT NULL,
    record_id VARCHAR(128) NOT NULL,
    command_id VARCHAR(32) NOT NULL,
    namespace VARCHAR(256) NOT NULL,
    subject VARCHAR(128) NOT NULL,
    payload TEXT NULL,
    recorded_at BIGINT NOT NULL,
    sequence BIGINT NOT NULL CHECK (sequence >= 0),
    PRIMARY KEY (tenant, record_id)
) ENGINE=InnoDB;

CREATE TABLE quent_dispatch_test.bundle_outbox (
    tenant BIGINT NOT NULL,
    record_id VARCHAR(128) NOT NULL,
    command_id VARCHAR(32) NOT NULL,
    namespace VARCHAR(256) NOT NULL,
    subject VARCHAR(128) NOT NULL,
    payload TEXT NULL,
    recorded_at BIGINT NOT NULL,
    sequence BIGINT NOT NULL CHECK (sequence >= 0),
    PRIMARY KEY (tenant, record_id)
) ENGINE=InnoDB;
