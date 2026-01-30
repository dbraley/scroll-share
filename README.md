# scroll-share

A multi-service TTRPG platform where players share structured documents (character sheets, session notes, etc.) with campaign members at different visibility levels.

## Architecture

```
Browser --> Next.js (BFF + UI) --> Backend Services --> PostgreSQL
                                       |
                          +------------+------------+-------------+
                          |            |            |             |
                     auth-service  campaign-svc  document-svc  permission-svc
                       (Rust)        (Go)        (Kotlin)       (Python)
```

### Services

| Service | Language/Framework | Port | Responsibilities |
|---------|-------------------|------|-----------------|
| `web` | Next.js / TypeScript | 3000 | UI, BFF API routes, SSR, auth cookie management |
| `auth-service` | Rust / Axum | 8081 | Registration, login, JWT issuance, user profiles |
| `campaign-service` | Go / chi | 8082 | Campaign CRUD, memberships, roles (GM/Player), invite codes |
| `document-service` | Kotlin / Ktor | 8083 | Sheet templates, document CRUD, versioning, field-level GM-only filtering |
| `permission-service` | Python / FastAPI | 8084 | Permission checks, sharing CRUD, visibility resolution |

### Key Concepts

- **Campaigns**: A GM creates a campaign and invites players via invite codes. All sharing is scoped within a campaign.
- **Documents**: Structured sheets (character sheets, NPCs, items) built from templates with typed fields, plus markdown support for text fields.
- **Visibility**: Documents can be private (owner + GM only), shared with specific campaign members, or visible to the entire campaign.
- **Templates**: Define the structure of a document type (e.g., "D&D 5e Character Sheet") with typed fields organized into sections. Fields can be marked `gm_only`.

## Tech Stack

- **Frontend**: Next.js (App Router), Tailwind CSS
- **Backend**: Polyglot microservices (Rust, Go, Kotlin, Python)
- **Database**: PostgreSQL 16 with schema-per-service isolation
- **Infrastructure**: Kubernetes, CloudNativePG, nginx-ingress
- **Local Dev**: kind (local K8s), Tilt (dev orchestration)
- **Auth**: Username/password with JWT (HS256), httpOnly cookies

## Project Structure

```
scroll-share/
  Tiltfile                           # Tilt dev orchestration
  docker-compose.dev.yaml            # Local Postgres for standalone dev
  services/
    auth-service/                    # Rust (Axum)
    campaign-service/                # Go (chi)
    document-service/                # Kotlin (Ktor)
    permission-service/              # Python (FastAPI)
  web/                               # Next.js frontend + BFF
  deploy/
    k8s/                             # Kubernetes manifests
    db/
      init/                          # DB schema/role initialization
```

## Getting Started

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/)
- [kind](https://kind.sigs.k8s.io/docs/user/quick-start/#installation)
- [Tilt](https://docs.tilt.dev/install.html)
- [kubectl](https://kubernetes.io/docs/tasks/tools/)

### Local Development

```bash
# Create a local K8s cluster
kind create cluster --name scroll-share

# Start all services with Tilt
tilt up
```

For standalone service development (without K8s):

```bash
# Start just Postgres
docker compose -f docker-compose.dev.yaml up -d

# Then run any individual service
cd services/auth-service && cargo watch -x run
cd services/campaign-service && go run .
cd services/document-service && gradle run
cd services/permission-service && uvicorn app.main:app --reload
cd web && npm run dev
```

## Implementation Phases

1. **Foundation** - Auth service, Next.js skeleton, K8s infrastructure, end-to-end login flow
2. **Campaigns** - Campaign CRUD, membership, invite codes, roles (GM/Player)
3. **Documents** - Sheet templates, document CRUD, versioning, editor UI
4. **Permissions & Sharing** - Visibility enforcement, sharing UI, GM overrides
5. **Polish** - Error handling, logging, OpenAPI specs, CI/CD

See [docs/architecture.md](docs/architecture.md) for the full design document.
