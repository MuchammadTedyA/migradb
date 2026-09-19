-- Migration: initial_schema
-- Created at: 2026-09-01T00:00:00Z

CREATE TABLE IF NOT EXISTS m_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_m_users_email ON m_users(email);
CREATE INDEX IF NOT EXISTS idx_m_users_username ON m_users(username);
