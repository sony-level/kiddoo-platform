-- This file should undo anything in `up.sql`
-- Rollback de la migration users

DROP TRIGGER IF EXISTS trigger_users_updated_at ON kidoo.users;
DROP FUNCTION IF EXISTS kidoo.update_updated_at_column();
DROP TABLE IF EXISTS kidoo.users;
DROP TYPE IF EXISTS kidoo.user_role;
DROP SCHEMA IF EXISTS kidoo CASCADE;

-- Ne pas supprimer les extensions (peuvent être utilisées ailleurs)