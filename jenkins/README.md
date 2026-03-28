# Jenkins CI/CD Configuration

## Overview

This directory contains the Jenkins **post-merge delivery pipeline** for the Kiddoo platform.

**GitHub Actions** validates PRs (lint, tests, secret scan).
**Jenkins** builds, scans, packages, and deploys (after merge).

## Files

- `Dockerfile.agent` - Build agent: Rust, AWS CLI, Docker, Trivy, SonarQube Scanner, Diesel CLI

## Combined CI/CD Strategy

```
GitHub Actions (before merge)          Jenkins (after merge)
─────────────────────────────          ──────────────────────────────────────
lint (fmt + clippy)                    build (cargo build --release)
unit tests + DB migrations             integration tests (real DB)
secret scan (Gitleaks)                 SonarQube (quality gate)
build check                            cargo-audit + Trivy (fs/config/image)
                                       Docker build & push (GHCR private)
                                       Promote to AWS ECR
                                       Deploy per environment
                                       Smoke tests / E2E / Health checks
```

## Pipeline Stages (Jenkins)

1. **Checkout** - Clone repo, extract version + commit
2. **Setup** - Verify toolchain (Rust, Diesel, Trivy)
3. **Database** - Start PostgreSQL container, run Diesel migrations
4. **Test** - `cargo test --workspace`
5. **Build** - `cargo build --release --workspace`
6. **Security & Quality** (parallel):
   - `cargo-audit` — dependency vulnerabilities
   - `Trivy filesystem` — source code scan
   - `Trivy config` — IaC/Dockerfile scan
7. **SonarQube** — code quality + quality gate (develop/release only)
8. **Docker Build** — build images locally
9. **Trivy Image Scan** — scan Docker images before push
10. **Push to GHCR** — push validated images privately
11. **Deploy** — promote to AWS ECR per environment
12. **Post-deploy tests** — smoke tests / E2E / health checks

## Branch → Environment Mapping

| Branch      | GitHub Actions   | Jenkins                   | Environment     | Deploy | Approval |
| ----------- | ---------------- | ------------------------- | --------------- | ------ | -------- |
| `feature/*` | lint, test, scan | -                         | -               | -      | -        |
| `develop`   | lint, test, scan | full pipeline             | **development** | Auto   | -        |
| `release/*` | lint, test, scan | full + SonarQube + E2E    | **staging**     | Manual | Required |
| `master`    | lint, test, scan | promotion + health checks | **production**  | Manual | Required |

## Required Jenkins Plugins

- Git Plugin
- Pipeline Plugin
- Docker Pipeline Plugin
- Discord Notifier Plugin
- Timestamps Plugin
- Credentials Binding Plugin
- SonarQube Scanner Plugin

## Required Jenkins Credentials

| Credential ID         | Type              | Description                            | Required |
| --------------------- | ----------------- | -------------------------------------- | -------- |
| `ghcr-token`          | Username/Password | GitHub username + PAT (packages:write) | Yes      |
| `aws-account-id`      | Secret text       | AWS Account ID (e.g. 123456789012)     | Yes      |
| `aws-ecr-credentials` | Username/Password | AWS Access Key ID + Secret Access Key  | Yes      |
| `sonarqube-url`       | Secret text       | SonarQube server URL                   | Yes      |
| `sonarqube-token`     | Secret text       | SonarQube authentication token         | Yes      |
| `discord-webhook-url` | Secret text       | Discord webhook URL for notifications  | Yes      |

## Required GitHub Secrets

| Secret Name           | Description                           |
| --------------------- | ------------------------------------- |
| `DISCORD_WEBHOOK_URL` | Discord webhook URL for notifications |

## GitHub Environments

Configure environments in **GitHub → Settings → Environments**:

### 1. `production`

- **Protection rule**: Required reviewers (at least 1 approver)
- **Branch restriction**: `master` only
- **Deploy**: Manual approval in Jenkins before deploy
- **Secrets**: `AWS_ACCOUNT_ID`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`

### 2. `staging`

- **Protection rule**: Required reviewers (at least 1 approver)
- **Branch restriction**: `release/*` only
- **Deploy**: Manual approval in Jenkins before deploy
- **Secrets**: `AWS_ACCOUNT_ID`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`

### 3. `development`

- **Protection rule**: None (auto-deploy on merge)
- **Branch restriction**: `develop` only
- **Deploy**: Auto on merge
- **Secrets**: `AWS_ACCOUNT_ID`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`

## Setup

### 1. Install required Jenkins plugins

Install all plugins listed above via Jenkins > Manage Jenkins > Plugins.

### 2. Create credentials in Jenkins

Go to Jenkins > Manage Jenkins > Credentials > System > Global credentials.

### 3. Create Multibranch Pipeline job

- New Item > Multibranch Pipeline
- Branch Sources > Git: `https://github.com/sony-level/api-babysiting.git`
- Build Configuration > Script Path: `Jenkinsfile`
- Scan Multibranch Pipeline Triggers: 1 minute or webhook

### 4. Configure GitHub webhook

In your GitHub repository settings:

- Settings > Webhooks > Add webhook
- Payload URL: `https://jenkins.level-sony.com/github-webhook/`
- Content type: `application/json`
- Events: Push events, Pull request events

## Docker Images

| Service        | GHCR (private)                             | AWS ECR (production)                                               |
| -------------- | ------------------------------------------ | ------------------------------------------------------------------ |
| API Gateway    | `ghcr.io/sony-level/kiddoo-api-gateway`    | `<account>.dkr.ecr.eu-north-1.amazonaws.com/kiddoo-api-gateway`    |
| Identity Proxy | `ghcr.io/sony-level/kiddoo-identity-proxy` | `<account>.dkr.ecr.eu-north-1.amazonaws.com/kiddoo-identity-proxy` |

### Image Tags

| Tag                 | Meaning                           |
| ------------------- | --------------------------------- |
| `<version>-<build>` | Versioned build (e.g. `0.1.0-42`) |
| `<commit-short>`    | Git commit SHA (e.g. `a1b2c3d`)   |
| `dev`               | Latest dev deployment             |
| `staging`           | Latest staging deployment         |
| `prod`              | Latest production deployment      |
| `latest`            | Latest production release         |
