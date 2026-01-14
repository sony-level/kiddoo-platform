-- This file should undo anything in `up.sql`
-- Rollback de la migration profiles

DROP POLICY IF EXISTS profiles_admin_all_access ON kidoo.profiles;
DROP POLICY IF EXISTS profiles_public_read ON kidoo.profiles;
DROP POLICY IF EXISTS profiles_own_full_access ON kidoo.profiles;

DROP TRIGGER IF EXISTS trigger_profiles_display_name ON kidoo.profiles;
DROP TRIGGER IF EXISTS trigger_profiles_updated_at ON kidoo.profiles;

DROP FUNCTION IF EXISTS kidoo.generate_display_name();

DROP TABLE IF EXISTS kidoo.profiles;
DROP TYPE IF EXISTS kidoo.kyc_status;