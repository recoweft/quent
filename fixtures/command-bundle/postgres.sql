-- Explicit application installation example for the dedicated test database.
-- Run as the installation owner BEFORE starting application command traffic.
-- No triggers, RLS, generated/default/identity columns or external effects.
CREATE TABLE quent_dispatch_test.bundle_items (
    tenant BIGINT NOT NULL,
    id BIGINT NOT NULL,
    amount BIGINT NOT NULL CHECK (amount >= 0),
    flag BOOLEAN NOT NULL,
    note TEXT NULL,
    PRIMARY KEY (tenant, id)
);
CREATE TABLE quent_dispatch_test.bundle_ledger (
    identity BYTEA PRIMARY KEY CHECK (octet_length(identity) BETWEEN 1 AND 2048),
    input BYTEA NOT NULL CHECK (octet_length(input) BETWEEN 4 AND 1048576),
    result BYTEA NULL CHECK (result IS NULL OR octet_length(result) = CASE WHEN version = 1 THEN 13 ELSE 29 END),
    version SMALLINT NOT NULL CHECK (version IN (1, 2))
);
-- Application role: SELECT, INSERT, UPDATE on these exact tables; no DELETE,
-- TRUNCATE, DDL or ledger rewrite privileges for untrusted callers.
-- The installation/retention operator must be separate from request callers.

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
);

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
);
