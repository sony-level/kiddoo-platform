# Environment Strategy — kiddoo-platform

## Overview

This document defines the 4 environments used in the project, their mapping to
Git branches, GitHub Environments, Kubernetes namespaces, and protection rules.

---

## Environment Matrix

| Environment | Git Branch | GitHub Environment | K8s Namespace | K8s Overlay | Protection |
|-------------|-----------|-------------------|---------------|-------------|------------|
| **Development** | `develop` | `development` | `kiddoo-dev` | `k8s/overlays/dev` | None (auto-deploy) |
| **Test** | any (manual) | `test` | `kiddoo-test` | `k8s/overlays/test` | None (on-demand) |
| **Staging** | `release/*` | `staging` | `kiddoo-staging` | `k8s/overlays/staging` | Required reviewers |
| **Production** | `master` | `production` | `kiddoo-production` | `k8s/overlays/production` | Required reviewers + wait timer |

---

## Flow Diagram

```
feature/* ──PR──> develop ──merge──> release/* ──merge──> master
                     │                    │                   │
                     ▼                    ▼                   ▼
              ┌────────────┐      ┌────────────┐      ┌────────────┐
              │    DEV     │      │  STAGING   │      │ PRODUCTION │
              │ (auto)     │      │ (approval) │      │ (approval) │
              └────────────┘      └────────────┘      └────────────┘

                                        ▲
                                        │ (independent)
                              ┌────────────────────┐
                              │       TEST         │
                              │ (manual dispatch)  │
                              └────────────────────┘
```

---

## Environment Details

### 1. Development (`dev`)

- **Purpose**: Continuous integration feedback for developers
- **Trigger**: Automatic on every push to `develop`
- **Replicas**: 1 per service
- **Log level**: `debug`
- **CORS**: `https://dev.level-sony.com`
- **Secrets path**: `kiddoo/dev/*` (AWS Secrets Manager)
- **ArgoCD app**: `kiddoo-dev`
- **CI workflow**: `cd.yml` → `deploy-dev` job

### 2. Test (`test`)

- **Purpose**: On-demand testing environment for QA / project leads
- **Trigger**: Manual via `workflow_dispatch` (Deploy Test Environment)
- **Replicas**: 1 per service
- **Log level**: `debug`
- **CORS**: `https://test.level-sony.com`
- **Secrets path**: `kiddoo/test/*` (AWS Secrets Manager)
- **ArgoCD app**: `kiddoo-test`
- **CI workflow**: `deploy-test.yml`
- **Key difference**: Can deploy ANY branch without affecting other environments

### 3. Staging (`staging`)

- **Purpose**: Pre-production validation for release candidates
- **Trigger**: Push to `release/*` branch (requires manual approval)
- **Replicas**: 2 per service
- **Log level**: `info`
- **CORS**: `https://staging.level-sony.com`
- **Secrets path**: `kiddoo/staging/*` (AWS Secrets Manager)
- **ArgoCD app**: `kiddoo-staging`
- **CI workflow**: `cd.yml` → `deploy-staging` job
- **Tests run**: E2E tests post-deployment

### 4. Production (`production`)

- **Purpose**: Live environment serving real users
- **Trigger**: Push to `master` branch (requires manual approval)
- **Replicas**: 3 per service
- **Log level**: `info` (no debug)
- **CORS**: `https://level-sony.com`
- **Secrets path**: `kiddoo/production/*` (AWS Secrets Manager)
- **ArgoCD app**: `kiddoo-production`
- **CI workflow**: `cd.yml` → `deploy-production` job
- **Post-deploy**: Health checks + automatic rollback if failed

---

## GitHub Environment Setup

### Required Environments (Settings > Environments)

Create these 4 environments in your GitHub repository settings:

#### `development`
- Protection rules: **None**
- Deployment branches: `develop` only

#### `test`
- Protection rules: **None**
- Deployment branches: All branches (any branch can be deployed for testing)

#### `staging`
- Protection rules: **Required reviewers** (1+ team leads)
- Deployment branches: `release/*` only
- Wait timer: Optional (0-30 min)

#### `production`
- Protection rules: **Required reviewers** (2+ team leads)
- Deployment branches: `master` only
- Wait timer: 5 minutes (safety buffer)

---

## AWS Secrets Manager Layout

```
kiddoo/
├── dev/
│   ├── database-url
│   ├── oauth-client-id
│   ├── oauth-client-secret
│   └── jwt-secret
├── test/
│   ├── database-url
│   ├── oauth-client-id
│   ├── oauth-client-secret
│   └── jwt-secret
├── staging/
│   ├── database-url
│   ├── oauth-client-id
│   ├── oauth-client-secret
│   └── jwt-secret
└── production/
    ├── database-url
    ├── oauth-client-id
    ├── oauth-client-secret
    └── jwt-secret
```

---

## ArgoCD Applications

| App Name | Path | Target Revision | Namespace |
|----------|------|-----------------|-----------|
| `kiddoo-dev` | `k8s/overlays/dev` | `develop` | `kiddoo-dev` |
| `kiddoo-test` | `k8s/overlays/test` | `HEAD` | `kiddoo-test` |
| `kiddoo-staging` | `k8s/overlays/staging` | `HEAD` | `kiddoo-staging` |
| `kiddoo-production` | `k8s/overlays/production` | `HEAD` | `kiddoo-production` |

---

## Differences Summary

| Aspect | Dev | Test | Staging | Production |
|--------|-----|------|---------|------------|
| Replicas | 1 | 1 | 2 | 3 |
| Log level | debug | debug | info | info |
| Auto-deploy | ✓ | ✓ (manual trigger) | ✗ (approval) | ✗ (approval) |
| Tests | — | smoke/integ/e2e | E2E | Health check |
| Rollback | Manual | Manual | Manual | Auto on failure |
| Isolation | Namespace | Namespace | Namespace | Namespace |

---

## Quick Reference

```bash
# Deploy to test environment (manual)
# GitHub > Actions > Deploy Test Environment > Run workflow

# Check pods per environment
kubectl get pods -n kiddoo-dev
kubectl get pods -n kiddoo-test
kubectl get pods -n kiddoo-staging
kubectl get pods -n kiddoo-production

# ArgoCD status
argocd app list | grep kiddoo

# Force sync
argocd app sync kiddoo-test
```
