# Test Environment Deployment & Testing Process

## Overview

This document describes the full process for preparing a test environment,
deploying a release candidate, running tests, and reporting failures
to the development team.

---

## 1. Retrieve application code

### Via Git (command line)

```bash
git clone https://github.com/sony-level/kiddoo-platform.git
cd kiddoo-platform
git checkout release/0.2.0
git log --oneline -5
```

### Via GitHub Actions (automated)

The `deploy-test.yml` workflow automatically checks out the code:

- **Manual trigger**: Actions > Deploy Test Environment > Run workflow
- **Parameters**:
  - `branch`: branch to deploy (e.g. `release/0.2.0`, `develop`)
  - `test_level`: `smoke`, `integration`, `e2e` or `full`
  - `notify_team`: notify the team via Discord

---

## 2. Create the test environment

### Architecture

```text
                   GitHub Actions
                        |
            +-----------+-----------+
            |                       |
       Build & Test          Security Scans
            |                  (Trivy, Audit)
            |
       Docker Build
            |
       Push to GHCR
            |
    Update K8s manifests
            |
       Git commit/push
            |
    ArgoCD detects change
            |
    Deploy to EKS (kiddoo-test)
            |
    +-------+-------+
    |       |       |
  api-gw  id-proxy  configmap/secrets
```

### Deployed components

| K8s Resource   | Description                               |
| -------------- | ----------------------------------------- |
| Namespace      | `kiddoo-test` (auto-created)              |
| Deployment     | `test-api-gateway` (1 replica)            |
| Deployment     | `test-identity-proxy` (1 replica)         |
| Service        | ClusterIP per service                     |
| ConfigMap      | Application config (log level, CORS, env) |
| ExternalSecret | Secrets from AWS Secrets Manager          |

### Specification compliance

The test environment mirrors production infrastructure:

- **Isolation**: dedicated Kubernetes namespace (`kiddoo-test`)
- **Configuration**: environment-specific variables
- **Secrets**: managed via AWS Secrets Manager (no plaintext secrets in Git)
- **Scalability**: 1 replica, matching the on-demand test overlay
- **Monitoring**: health checks (liveness + readiness probes)
- **Rolling update**: zero-downtime deployments

---

## 3. Produce a test version

### Automatic trigger (CD Pipeline)

On every merge to a `release/*` branch:

1. **ci.yml** (PR): lint, unit tests, secret scan
2. **cd.yml** (post-merge):
   - Rust build (`cargo build --release`)
   - Unit tests (`cargo test`)
   - Security audit (`cargo audit`)
   - Trivy scan (filesystem + config + images)
   - SonarQube analysis
   - Docker build + push to GHCR
   - **Manual approval** (GitHub Environment `staging`)
   - Update K8s manifests
   - ArgoCD auto-sync

### Manual trigger

```text
GitHub > Actions > Deploy Test Environment > Run workflow
  - branch: release/0.2.0
  - test_level: full
  - notify_team: true
```

### Versioning

Format: `{version}-{run_number}` (e.g. `0.1.0-42`)

- Version extracted from `Cargo.toml`
- GitHub Actions run number for uniqueness

---

## 4. Run tests

### Test levels

| Level           | Description                 | When                    |
| --------------- | --------------------------- | ----------------------- |
| **Smoke**       | Endpoint health checks      | After every deployment  |
| **Integration** | Unit + DB integration tests | Build stage             |
| **E2E**         | Full user scenarios         | Post-deployment staging |

### Smoke tests

Verify services respond correctly:

- `GET /api/v1/health` on api-gateway (HTTP 200)
- `GET /api/v1/health` on identity-proxy (HTTP 200)
- Response time < 2 seconds

### Integration tests

Run with PostgreSQL as a service container:

```bash
cargo test --workspace --no-fail-fast
```

### End-to-End tests

Full functional scenarios:

1. New user registration
2. Authentication (login + JWT)
3. Access protected route with JWT
4. Reject access without token
5. Swagger UI availability

### Viewing results

- **GitHub Actions**: "Summary" tab for structured reports
- **Artifacts**: downloadable test files (unit-test-results, integration-test-results)
- **Discord**: automatic notification with status

---

## 5. Report failures to developers

### Automatic reporting

On test failure, the pipeline automatically creates a **GitHub Issue** with:

- Tested version and source branch
- Failed test suites
- Link to the run logs
- Corrective action checklist
- Labels: `bug`, `test-environment`, `needs-fix`

### Manual reporting

Use the dedicated issue template:

```text
GitHub > Issues > New Issue > "Bug Report - Test Environment"
```

The form requests:

- Tested version
- Affected service (api-gateway, identity-proxy, etc.)
- Severity (critical, major, minor, cosmetic)
- Test type (smoke, integration, E2E, manual)
- Description (expected vs observed behavior)
- Steps to reproduce
- Logs / error traces

### Fix cycle

```text
Failure detected
        |
  GitHub Issue created (automatic or manual)
        |
  Developer assigned
        |
  Fix on release/* branch
        |
  Push -> CI validates -> CD re-deploys
        |
  Tests re-executed
        |
  Issue closed if tests pass
```

---

## Useful commands

### Check deployment status

```bash
# Pod status
kubectl get pods -n kiddoo-test

# Service logs
kubectl logs -n kiddoo-test deployment/test-api-gateway

# ArgoCD status
argocd app get kiddoo-test

# Force ArgoCD sync
argocd app sync kiddoo-test
```

### Rollback

```bash
# Via ArgoCD (revert to previous version)
argocd app rollback kiddoo-test

# Via Kubernetes
kubectl rollout undo deployment/test-api-gateway -n kiddoo-test
kubectl rollout undo deployment/test-identity-proxy -n kiddoo-test
```

### Run tests locally

```bash
# Unit + integration tests
docker compose up -d postgres
export DATABASE_URL=postgres://kiddoo:kiddoo_test_pwd@localhost:5433/kiddoo_test
cd libs/database && diesel migration run && cd ../..
cargo test --workspace

# Build release
cargo build --release --workspace
```

---

## Reference files

| File                                              | Purpose                                                |
| ------------------------------------------------- | ------------------------------------------------------ |
| `.github/workflows/ci.yml`                        | PR validation (lint, tests, secret scan)               |
| `.github/workflows/cd.yml`                        | CD pipeline (build, Docker, deploy, post-deploy tests) |
| `.github/workflows/deploy-test.yml`               | On-demand test environment deployment                  |
| `.github/ISSUE_TEMPLATE/bug-test-environment.yml` | Issue template for bug reports                         |
| `k8s/overlays/test/`                              | K8s manifests for the test environment                 |
| `k8s/overlays/staging/`                           | K8s manifests for staging (release validation)         |
| `k8s/argocd/test.yaml`                            | ArgoCD Application for test environment                |
| `k8s/argocd/staging.yaml`                         | ArgoCD Application for staging                         |
| `docs/environments.md`                            | Full environment strategy reference                    |
