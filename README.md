# Kiddoo Platform

REST API for the Kiddoo babysitting mobile application.

## Tech Stack

- **Language**: Rust (Rocket framework)
- **Database**: PostgreSQL + Diesel ORM
- **CI/CD**: GitHub Actions
- **Registry**: GitHub Container Registry (GHCR)
- **Deployment**: Kubernetes (EKS) + ArgoCD
- **Quality**: SonarQube, Trivy, CodeQL

## Services

| Service          | Port | Description               |
| ---------------- | ---- | ------------------------- |
| `api-gateway`    | 8000 | Main API gateway          |
| `identity-proxy` | 8001 | Authentication & identity |
| `orchestrator`   | —    | Service orchestration     |

## CI/CD Workflows

| Workflow          | Trigger                                  | Role                       |
| ----------------- | ---------------------------------------- | -------------------------- |
| `ci.yml`          | PR                                       | Lint, test, secret scan    |
| `cd.yml`          | Push on `develop`, `release/*`, `master` | Build, Docker, deploy      |
| `deploy-test.yml` | Manual                                   | On-demand test environment |
| `codeql.yml`      | Push, PR, weekly                         | Security analysis          |

## Environments

The environment matrix, GitHub Environment rules, Kubernetes namespaces, ArgoCD
apps, and AWS Secrets Manager layout are centralized in
[docs/environments.md](docs/environments.md).

## Required Secrets

### GitHub Secrets (Settings > Secrets and variables > Actions)

| Secret                  | Description                           | Required  |
| ----------------------- | ------------------------------------- | --------- |
| `SONAR_HOST_URL`        | SonarQube server URL                  | Yes       |
| `SONAR_TOKEN`           | SonarQube auth token                  | Yes       |
| `AWS_ACCESS_KEY_ID`     | AWS IAM key (start/stop SonarQube VM) | Yes       |
| `AWS_SECRET_ACCESS_KEY` | AWS IAM secret                        | Yes       |
| `SONAR_EC2_INSTANCE_ID` | EC2 instance ID hosting SonarQube     | Yes       |
| `DISCORD_WEBHOOK_URL`   | Discord notification webhook          | Optional  |
| `GITHUB_TOKEN`          | Provided automatically by GitHub      | Automatic |

GitHub Environments, application secrets, and Kubernetes bootstrap details are
documented in [docs/environments.md](docs/environments.md).

## Local Development

```bash
# Switch to the desired environment
make env ENV=development   # or test, staging, production

# Start database
docker compose up -d postgres

# Run migrations
export DATABASE_URL=postgres://kiddoo:kiddoo_test_pwd@localhost:5433/kiddoo_test
cd libs/database && diesel migration run && cd ../..

# Run tests
cargo test --workspace

# Build
cargo build --release --workspace
```

## Project Structure

```text
├── services/
│   ├── api-gateway/          # Main API service
│   ├── identity-proxy/       # Auth service
│   └── orchestrator/         # Orchestration service
├── libs/
│   └── database/             # Shared DB layer (Diesel)
├── k8s/
│   ├── base/                 # K8s base manifests
│   ├── overlays/             # Kustomize per-env patches
│   └── argocd/               # ArgoCD Application manifests
├── .github/
│   ├── workflows/            # CI/CD pipelines
│   └── ISSUE_TEMPLATE/       # Issue templates
└── docs/                     # Documentation
```
