-- Your SQL goes here
-- ============================================================================
-- MIGRATION: Create profiles table
-- Description: Profils détaillés des utilisateurs
-- ============================================================================

-- Type énuméré pour KYC status
CREATE TYPE kidoo.kyc_status AS ENUM (
    'pending',
    'submitted',
    'under_review',
    'verified',
    'rejected',
    'expired'
);

-- Table profiles
CREATE TABLE kidoo.profiles (
    -- Identifiants
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID UNIQUE NOT NULL REFERENCES kidoo.users(id) ON DELETE CASCADE,
    
    -- Informations personnelles
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    display_name VARCHAR(200),
    date_of_birth DATE,
    gender VARCHAR(20),
    
    -- Photo de profil
    profile_picture_url TEXT,
    profile_picture_hash VARCHAR(64),
    
    -- Présentation
    bio TEXT,
    hourly_rate_cents INTEGER,
    
    -- KYC (Know Your Customer)
    kyc_status kidoo.kyc_status DEFAULT 'pending' NOT NULL,
    kyc_verified_at TIMESTAMPTZ,
    kyc_verified_by UUID REFERENCES kidoo.users(id),
    kyc_notes TEXT,
    kyc_expires_at DATE,
    
    -- Données spécifiques par rôle (JSONB)
    role_specific_data JSONB DEFAULT '{}'::jsonb NOT NULL,
    
    -- Statistiques
    total_bookings INTEGER DEFAULT 0 NOT NULL,
    completed_bookings INTEGER DEFAULT 0 NOT NULL,
    average_rating NUMERIC(3,2) DEFAULT 0.00,
    total_reviews INTEGER DEFAULT 0 NOT NULL,
    cancellation_rate NUMERIC(5,2) DEFAULT 0.00,
    
    -- Visibilité
    is_visible BOOLEAN DEFAULT TRUE NOT NULL,
    is_featured BOOLEAN DEFAULT FALSE NOT NULL,
    search_rank INTEGER DEFAULT 0 NOT NULL,
    
    -- Métadonnées
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    
    -- Contraintes
    CONSTRAINT hourly_rate_positive CHECK (
        hourly_rate_cents IS NULL OR hourly_rate_cents >= 500
    ),
    CONSTRAINT hourly_rate_maximum CHECK (
        hourly_rate_cents IS NULL OR hourly_rate_cents <= 10000
    ),
    CONSTRAINT rating_range CHECK (
        average_rating >= 0 AND average_rating <= 5
    ),
    CONSTRAINT cancellation_rate_range CHECK (
        cancellation_rate >= 0 AND cancellation_rate <= 100
    ),
    CONSTRAINT adult_age CHECK (
        date_of_birth IS NULL OR 
        date_of_birth <= CURRENT_DATE - INTERVAL '18 years'
    )
);

-- Index pour performances
CREATE UNIQUE INDEX idx_profiles_user_id ON kidoo.profiles(user_id);

CREATE INDEX idx_profiles_kyc_status ON kidoo.profiles(kyc_status) 
    WHERE kyc_status != 'verified';

CREATE INDEX idx_profiles_search ON kidoo.profiles(is_visible, search_rank DESC) 
    WHERE is_visible = TRUE;

CREATE INDEX idx_profiles_rating ON kidoo.profiles(average_rating DESC NULLS LAST, total_reviews DESC);

CREATE INDEX idx_profiles_names_trgm ON kidoo.profiles 
    USING gin((first_name || ' ' || last_name) gin_trgm_ops);

CREATE INDEX idx_profiles_display_name_trgm ON kidoo.profiles 
    USING gin(display_name gin_trgm_ops) 
    WHERE display_name IS NOT NULL;

CREATE INDEX idx_profiles_role_data ON kidoo.profiles USING gin(role_specific_data);

-- Fonction génération display_name
CREATE OR REPLACE FUNCTION kidoo.generate_display_name()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.display_name IS NULL AND NEW.first_name IS NOT NULL THEN
        NEW.display_name := NEW.first_name || ' ' || LEFT(NEW.last_name, 1) || '.';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers
CREATE TRIGGER trigger_profiles_updated_at
    BEFORE UPDATE ON kidoo.profiles
    FOR EACH ROW
    EXECUTE FUNCTION kidoo.update_updated_at_column();

CREATE TRIGGER trigger_profiles_display_name
    BEFORE INSERT OR UPDATE ON kidoo.profiles
    FOR EACH ROW
    EXECUTE FUNCTION kidoo.generate_display_name();

-- Row-Level Security
ALTER TABLE kidoo.profiles ENABLE ROW LEVEL SECURITY;

-- Politique: Un utilisateur voit son propre profil
CREATE POLICY profiles_own_full_access ON kidoo.profiles
    FOR ALL
    USING (user_id = current_setting('app.current_user_id', TRUE)::UUID);

-- Politique: Profils publics visibles par tous
CREATE POLICY profiles_public_read ON kidoo.profiles
    FOR SELECT
    USING (
        is_visible = TRUE AND 
        EXISTS (
            SELECT 1 FROM kidoo.users 
            WHERE users.id = profiles.user_id 
            AND users.is_active = TRUE 
            AND users.deleted_at IS NULL
        )
    );

-- Politique: moderators ont accès à tout
CREATE POLICY profiles_moderator_all_access ON kidoo.profiles
    FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM kidoo.users 
            WHERE users.id = current_setting('app.current_user_id', TRUE)::UUID 
            AND users.role = 'moderator'
        )
    );

-- Commentaires
COMMENT ON TABLE kidoo.profiles IS 
'Profils détaillés des utilisateurs, séparés de users pour performances et sécurité';

COMMENT ON COLUMN kidoo.profiles.role_specific_data IS 
'Données JSONB spécifiques au rôle pour flexibilité';

COMMENT ON COLUMN kidoo.profiles.search_rank IS 
'Score calculé pour algorithme de matching';