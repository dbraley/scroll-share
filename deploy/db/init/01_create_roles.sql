-- Per-service database roles
-- Each service connects with its own user and can only access its schema

-- Auth service user
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'auth_user') THEN
        CREATE USER auth_user WITH PASSWORD 'auth_dev_password';
    END IF;
END
$$;
GRANT ALL PRIVILEGES ON SCHEMA auth TO auth_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA auth GRANT ALL ON TABLES TO auth_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA auth GRANT ALL ON SEQUENCES TO auth_user;

-- Campaign service user
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'campaign_user') THEN
        CREATE USER campaign_user WITH PASSWORD 'campaign_dev_password';
    END IF;
END
$$;
GRANT ALL PRIVILEGES ON SCHEMA campaign TO campaign_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA campaign GRANT ALL ON TABLES TO campaign_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA campaign GRANT ALL ON SEQUENCES TO campaign_user;

-- Document service user
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'document_user') THEN
        CREATE USER document_user WITH PASSWORD 'document_dev_password';
    END IF;
END
$$;
GRANT ALL PRIVILEGES ON SCHEMA document TO document_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA document GRANT ALL ON TABLES TO document_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA document GRANT ALL ON SEQUENCES TO document_user;

-- Permission service user
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'permission_user') THEN
        CREATE USER permission_user WITH PASSWORD 'permission_dev_password';
    END IF;
END
$$;
GRANT ALL PRIVILEGES ON SCHEMA permission TO permission_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA permission GRANT ALL ON TABLES TO permission_user;
ALTER DEFAULT PRIVILEGES IN SCHEMA permission GRANT ALL ON SEQUENCES TO permission_user;
