-- ============================================================================
-- MIGRATION: Create users table
-- Description: Table principale des comptes utilisateurs avec Keycloak
-- ============================================================================

-- Extensions PostgreSQL nécessaires
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Création du schéma
CREATE SCHEMA IF NOT EXISTS kidoo;

-- Type énuméré pour les rôles
CREATE TYPE kidoo.user_role AS ENUM (
    'parent',
    'babysitter',
    'agency',
    'moderator'
);

-- Table users
CREATE TABLE kidoo.users (
    -- Identifiants
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    keycloak_id UUID UNIQUE NOT NULL,
    
    -- Contact
    email VARCHAR(255) UNIQUE NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE NOT NULL,
    phone VARCHAR(20),
    phone_verified BOOLEAN DEFAULT FALSE NOT NULL,
    
    -- Rôle et statut
    role kidoo.user_role NOT NULL,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    is_blocked BOOLEAN DEFAULT FALSE NOT NULL,
    blocked_reason TEXT,
    blocked_at TIMESTAMPTZ,
    blocked_by UUID,
    
    -- Préférences
    locale VARCHAR(5) DEFAULT 'fr_FR' NOT NULL,
    timezone VARCHAR(50) DEFAULT 'Europe/Paris' NOT NULL,
    theme VARCHAR(20) DEFAULT 'auto' NOT NULL,
    
    -- Consentements RGPD
    gdpr_consent BOOLEAN DEFAULT FALSE NOT NULL,
    gdpr_consent_date TIMESTAMPTZ,
    gdpr_consent_version VARCHAR(10),
    marketing_consent BOOLEAN DEFAULT FALSE NOT NULL,
    marketing_consent_date TIMESTAMPTZ,
    
    -- Métadonnées
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    last_login_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    deletion_reason TEXT,
    
    -- Contraintes
    CONSTRAINT email_format CHECK (
        email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'
    ),
    CONSTRAINT phone_format CHECK (
        phone IS NULL OR phone ~* '^\+?[0-9]{8,15}$'
    ),
    CONSTRAINT locale_valid CHECK (
        locale IN ('fr_FR', 'en_US', 'en_GB', 'es_ES')
    ),
    CONSTRAINT valid_blocked_state CHECK (
        (is_blocked = FALSE AND blocked_reason IS NULL) OR
        (is_blocked = TRUE AND blocked_reason IS NOT NULL AND blocked_at IS NOT NULL)
    )
);

-- Index pour performances
CREATE INDEX idx_users_keycloak_id ON kidoo.users(keycloak_id) 
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_email ON kidoo.users(email) 
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_role ON kidoo.users(role) 
    WHERE deleted_at IS NULL AND is_active = TRUE;

CREATE INDEX idx_users_active_status ON kidoo.users(is_active, is_blocked) 
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_created_at ON kidoo.users(created_at DESC);

CREATE INDEX idx_users_email_trgm ON kidoo.users USING gin(email gin_trgm_ops);

-- Fonction trigger pour updated_at
CREATE OR REPLACE FUNCTION kidoo.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger sur users
CREATE TRIGGER trigger_users_updated_at
    BEFORE UPDATE ON kidoo.users
    FOR EACH ROW
    EXECUTE FUNCTION kidoo.update_updated_at_column();

-- Commentaires
COMMENT ON TABLE kidoo.users IS 
'Table principale des comptes utilisateurs avec intégration Keycloak SSO';

COMMENT ON COLUMN kidoo.users.keycloak_id IS 
'UUID Keycloak pour SSO. Keycloak gère mot de passe et MFA.';

COMMENT ON COLUMN kidoo.users.gdpr_consent IS 
'Consentement RGPD obligatoire pour utiliser la plateforme';