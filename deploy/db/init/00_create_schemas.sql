-- Schema creation for scroll-share services
-- Each service has its own schema for isolation

CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS campaign;
CREATE SCHEMA IF NOT EXISTS document;
CREATE SCHEMA IF NOT EXISTS permission;

COMMENT ON SCHEMA auth IS 'Auth service: users, sessions, tokens';
COMMENT ON SCHEMA campaign IS 'Campaign service: campaigns, members, invitations';
COMMENT ON SCHEMA document IS 'Document service: templates, documents, versions';
COMMENT ON SCHEMA permission IS 'Permission service: shares, visibility caches';
