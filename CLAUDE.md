# CLAUDE.md

## Project Overview

scroll-share is a multi-service TTRPG platform for sharing structured documents (character sheets, session notes, etc.) within campaigns. See [docs/architecture.md](docs/architecture.md) for the full design.

## Development Workflow

All development follows a strict BDD/TDD cycle. No exceptions.

### Feature Branch Flow

1. **Create a branch** for the feature
2. **Define the behavioral change** — write a clear, specific description of the desired behavior
3. **Write a BDD test** that captures the behavior (it will fail)
4. **Plan the implementation** — determine how to break the work into small functional pieces
5. **TDD inner loop** (repeat until the BDD test passes):
   a. **Red** — write a single unit test that drives toward making the BDD test pass (it will fail)
   b. **Green** — write the minimum code to make that test pass
   c. **Refactor** — improve the code OR the test, but **never both at the same time**. After each refactor step, all tests must still pass.
   d. Return to (a) for the next unit test
6. **BDD test passes** — if more behavior is needed, write another BDD test and repeat from step 5
7. **Feature complete** — open a PR

### Key Rules

- Never write production code without a failing test driving it
- Refactor steps change code or tests, never both simultaneously
- Every refactor step must end with all tests passing
- BDD tests describe behavior from the outside; TDD tests drive internal implementation
- Commits should be small and frequent, aligned with the red/green/refactor steps

## Services

| Service | Language | Port | Run Standalone |
|---------|----------|------|----------------|
| `web` | TypeScript (Next.js) | 3000 | `cd web && npm run dev` |
| `auth-service` | Rust (Axum) | 8081 | `cd services/auth-service && cargo watch -x run` |
| `campaign-service` | Go (chi) | 8082 | `cd services/campaign-service && go run .` |
| `document-service` | Kotlin (Ktor) | 8083 | `cd services/document-service && gradle run` |
| `permission-service` | Python (FastAPI) | 8084 | `cd services/permission-service && uvicorn app.main:app --reload` |

## Testing Commands by Language

### Rust (auth-service)
```bash
cd services/auth-service
cargo test              # Run all tests
cargo test <name>       # Run specific test
```

### Go (campaign-service)
```bash
cd services/campaign-service
go test ./...           # Run all tests
go test -run <name>     # Run specific test
```

### Kotlin (document-service)
```bash
cd services/document-service
gradle test             # Run all tests
gradle test --tests <name>  # Run specific test
```

### Python (permission-service)
```bash
cd services/permission-service
pytest                  # Run all tests
pytest -k <name>        # Run specific test
```

### Next.js (web)
```bash
cd web
npm test                # Run all tests
npm test -- <pattern>   # Run specific test
```

## Infrastructure

```bash
# Local K8s cluster
kind create cluster --name scroll-share

# Start all services
tilt up

# Standalone Postgres only
docker compose -f docker-compose.dev.yaml up -d
```

## Database

Single PostgreSQL instance with schema-per-service: `auth`, `campaign`, `document`, `permission`. Each service has its own DB user and manages its own migrations.

## Project Structure

```
scroll-share/
  services/
    auth-service/          Rust (Axum)
    campaign-service/      Go (chi)
    document-service/      Kotlin (Ktor)
    permission-service/    Python (FastAPI)
  web/                     Next.js
  deploy/
    k8s/                   Kubernetes manifests
    db/init/               Schema and role setup
```
