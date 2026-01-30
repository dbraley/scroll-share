# Architecture

## Services

### web (Next.js / TypeScript - port 3000)

The frontend and Backend for Frontend (BFF) layer. The only service exposed externally via Ingress.

- Server-side rendered UI with Next.js App Router
- API routes at `/api/*` proxy requests to backend services
- Manages auth cookies (httpOnly/Secure/SameSite=Strict)
- Converts cookies to `Authorization: Bearer` headers when calling backend services
- Handles token refresh automatically on 401 responses

### auth-service (Rust / Axum - port 8081)

Handles user identity and authentication.

- User registration with Argon2id password hashing
- Login with JWT issuance (access token: 15min, refresh token: 7 days)
- Token refresh endpoint
- User profile retrieval and batch lookup
- Health/readiness endpoints

### campaign-service (Go / chi - port 8082)

Manages campaigns, memberships, and roles.

- Campaign CRUD (create, read, update, archive)
- Membership management (invite, accept, remove, list)
- Role management (GM vs Player)
- Invite code generation and validation (with expiry and max uses)

### document-service (Kotlin / Ktor - port 8083)

The most complex service. Manages templates, documents, and versioning.

- Sheet template CRUD with JSON schema for typed fields
- Document CRUD (create from template, freeform, or both)
- Document versioning (each save creates a new version)
- GM-only field stripping based on requester role
- Calls permission-service for read authorization

### permission-service (Python / FastAPI - port 8084)

Centralized permission checks and sharing management.

- Single-document and batch permission checks
- Campaign-scoped document listing (what can user X see in campaign Y?)
- Sharing CRUD (grant/revoke access to specific users)
- Visibility updates
- Maintains denormalized caches of document visibility and campaign membership

## Data Model

### Database Strategy

Single PostgreSQL 16 instance with **schema-per-service** isolation:

- `auth` schema: owned by `auth_user` DB role
- `campaign` schema: owned by `campaign_user` DB role
- `document` schema: owned by `document_user` DB role
- `permission` schema: owned by `permission_user` DB role

Each service connects with its own credentials and can only access its own schema.

### Entity Relationships

```
users ──< campaign_members >── campaigns
  │                               │
  │                               ├──< campaign_invitations
  │                               │
  │                               ├──< sheet_templates
  │                               │
  └──< documents ────────────────>┘
       │
       ├──< document_versions
       │
       └──< document_shares >── users
```

### Tables

#### auth.users
| Column | Type | Notes |
|--------|------|-------|
| id | UUID | PK |
| username | VARCHAR(50) | Unique |
| display_name | VARCHAR(100) | |
| password_hash | VARCHAR(255) | Argon2id |
| created_at | TIMESTAMPTZ | |
| updated_at | TIMESTAMPTZ | |

#### campaign.campaigns
| Column | Type | Notes |
|--------|------|-------|
| id | UUID | PK |
| name | VARCHAR(200) | |
| description | TEXT | |
| created_by | UUID | FK -> users |
| game_system | VARCHAR(100) | e.g., "D&D 5e" |
| created_at | TIMESTAMPTZ | |
| updated_at | TIMESTAMPTZ | |
| archived_at | TIMESTAMPTZ | Soft archive |

#### campaign.campaign_members
| Column | Type | Notes |
|--------|------|-------|
| id | UUID | PK |
| campaign_id | UUID | FK -> campaigns |
| user_id | UUID | FK -> users |
| role | VARCHAR(20) | 'gm' or 'player' |
| joined_at | TIMESTAMPTZ | |

UNIQUE(campaign_id, user_id)

#### campaign.campaign_invitations
| Column | Type | Notes |
|--------|------|-------|
| id | UUID | PK |
| campaign_id | UUID | FK -> campaigns |
| invite_code | VARCHAR(20) | Unique |
| created_by | UUID | FK -> users |
| expires_at | TIMESTAMPTZ | |
| max_uses | INT | Default 1 |
| uses | INT | Default 0 |
| created_at | TIMESTAMPTZ | |

#### document.sheet_templates
| Column | Type | Notes |
|--------|------|-------|
| id | UUID | PK |
| name | VARCHAR(200) | e.g., "D&D 5e Character Sheet" |
| game_system | VARCHAR(100) | |
| schema | JSONB | Field definitions |
| created_by | UUID | FK -> users |
| campaign_id | UUID | FK -> campaigns, NULL = system template |
| created_at | TIMESTAMPTZ | |
| updated_at | TIMESTAMPTZ | |

#### document.documents
| Column | Type | Notes |
|--------|------|-------|
| id | UUID | PK |
| campaign_id | UUID | FK -> campaigns |
| template_id | UUID | FK -> sheet_templates, NULL for freeform |
| owner_id | UUID | FK -> users |
| title | VARCHAR(300) | |
| doc_type | VARCHAR(50) | 'character_sheet', 'note', 'session_log', 'npc', 'item' |
| visibility | VARCHAR(20) | 'private', 'shared', 'campaign' |
| current_version | INT | |
| created_at | TIMESTAMPTZ | |
| updated_at | TIMESTAMPTZ | |

#### document.document_versions
| Column | Type | Notes |
|--------|------|-------|
| id | UUID | PK |
| document_id | UUID | FK -> documents |
| version_number | INT | |
| field_data | JSONB | Structured field values |
| markdown_body | TEXT | Freeform markdown content |
| created_at | TIMESTAMPTZ | |

UNIQUE(document_id, version_number)

#### permission.document_shares
| Column | Type | Notes |
|--------|------|-------|
| id | UUID | PK |
| document_id | UUID | FK -> documents |
| shared_with | UUID | FK -> users |
| permission | VARCHAR(20) | 'view', 'comment' |
| created_at | TIMESTAMPTZ | |

UNIQUE(document_id, shared_with)

### Template Schema Format

The `schema` JSONB in `sheet_templates` defines the structure of documents:

```json
{
  "sections": [
    {
      "name": "Section Name",
      "fields": [
        {
          "key": "field_key",
          "label": "Display Label",
          "type": "text|number|checkbox|select|multiselect|date|markdown|list",
          "required": false,
          "gm_only": false,
          "min": null,
          "max": null,
          "options": [],
          "item_schema": { "fields": [] }
        }
      ]
    }
  ]
}
```

**Field types**:
- `text` - Single-line text input
- `number` - Numeric input with optional min/max
- `checkbox` - Boolean toggle
- `select` - Single selection from options list
- `multiselect` - Multiple selections from options list
- `date` - Date picker
- `markdown` - Multi-line text with markdown rendering
- `list` - Repeatable group of sub-fields (e.g., inventory items)

**Special flags**:
- `gm_only: true` - Field is only visible to the document owner and the campaign GM
- `required: true` - Field must have a value

## Auth Flow

### JWT Strategy

- Algorithm: HS256 (symmetric key shared across services via K8s Secret)
- Access token: 15-minute expiry, contains `{ sub: user_id, username, iat, exp }`
- Refresh token: 7-day expiry, stored in DB for revocation

### Flow

1. Browser sends credentials to Next.js BFF (`POST /api/auth/login`)
2. BFF proxies to auth-service
3. Auth-service validates credentials, returns JWT pair
4. BFF stores tokens in httpOnly/Secure/SameSite=Strict cookies
5. On subsequent requests, BFF reads cookie and adds `Authorization: Bearer` header to backend calls
6. Each backend service validates JWT locally using the shared signing key
7. On 401, BFF automatically refreshes the token and retries

## Permissions Model

### Visibility Levels

| Level | Who can see |
|-------|------------|
| `private` | Document owner + campaign GM |
| `shared` | Document owner + users in `document_shares` + campaign GM |
| `campaign` | All campaign members |

### Check Logic (priority order)

1. User is document owner -> **ALLOW**
2. User is GM of the document's campaign -> **ALLOW**
3. Visibility is `campaign` AND user is a campaign member -> **ALLOW**
4. Visibility is `shared` AND user appears in `document_shares` -> **ALLOW**
5. **DENY**

### Write Access

Only document owners can edit their documents. GMs can see everything but cannot edit other players' documents.

## Infrastructure

### Kubernetes Resources (namespace: scroll-share)

- **Deployments**: web, auth-service, campaign-service, document-service, permission-service (1-2 replicas each)
- **Services**: ClusterIP for each deployment
- **Ingress**: nginx-ingress routing all traffic to web service
- **StatefulSet/Operator**: CloudNativePG for PostgreSQL 16
- **Secrets**: JWT signing key, per-service DB credentials
- **ConfigMap**: Service URLs, log levels, non-secret configuration

### Local Development

- **kind**: Local Kubernetes cluster
- **Tilt**: Build/deploy orchestration with live reload
- **docker-compose.dev.yaml**: Standalone Postgres for single-service development

### Database Migrations

Each service manages its own migrations:

| Service | Migration Tool |
|---------|---------------|
| auth-service (Rust) | sqlx-cli |
| campaign-service (Go) | golang-migrate |
| document-service (Kotlin) | Flyway |
| permission-service (Python) | Alembic |

## API Design

### External API (Browser -> BFF)

REST endpoints served by Next.js API routes. See [api.md](api.md) for the full API reference.

### Internal API (Service -> Service)

REST/JSON over Kubernetes ClusterIP services. Each service publishes an OpenAPI spec.

All internal requests include an `Authorization: Bearer <jwt>` header. Each service validates the JWT in middleware before processing requests.
