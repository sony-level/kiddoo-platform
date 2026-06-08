# Supervision et Statistiques de Services — Kiddoo Platform

## 1. URLs des outils de supervision

| Outil      | URL                                 | Role                               |
| ---------- | ----------------------------------- | ---------------------------------- |
| Prometheus | https://prometheus.level-sony.cloud | Collecte et stockage des metriques |
| Grafana    | https://grafana.level-sony.cloud    | Visualisation et dashboards        |
| ArgoCD     | https://argocd.level-sony.cloud     | Deploiement GitOps                 |

## 2. Architecture de supervision

> **Infrastructure** :
>
> - **ArgoCD** est heberge sur un **serveur cloud dedie** (argocd.level-sony.cloud)
> - **Prometheus + Grafana** sont deployes sur le **serveur distant** (cluster Kubernetes applicatif)
> - **Aucune installation locale** n'est necessaire
> - ArgoCD surveille le repo GitHub et deploie les manifestes sur le cluster applicatif distant

```
  Poste developpeur            Cloud (ArgoCD)              Serveur distant (K8s cluster)
  +------------------+    +---------------------+    +-----------------------------------+
  |                  |    |                     |    |                                   |
  |  git push ----------->|  GitHub             |    |  +----------------+               |
  |  code + manifests|    |  sony-level/        |    |  | Kiddoo Pods    |               |
  |                  |    |  kiddoo-platform    |    |  | api-gw:8000    |               |
  +------------------+    |         |           |    |  | id-proxy:8001  |               |
                          |         v           |    |  |  /metrics      |               |
  +------------------+    |  +------+--------+  |    |  +-------+--------+               |
  |                  |    |  |   ArgoCD      |  |    |          |                        |
  |  Consulter       |    |  | argocd.level- +-------->  deploy manifestes              |
  |  dashboards      |    |  |  sony.cloud   |  |    |          |                        |
  |  via navigateur  |    |  +--------------+  |    |          v  scrape /metrics (15s) |
  |                  |    +---------------------+    |  +-------+--------+               |
  |             ------------------------------------>|  |  Prometheus    |  +----------+ |
  |                  |                               |  |  prometheus.  --->| AlertMgr | |
  |             ------------------------------------>|  |  level-sony   |  +-----+----+ |
  |                  |                               |  |  .cloud       |        |      |
  +------------------+                               |  +-------+-------+  Slack/Discord|
                                                     |          |                        |
                                                     |          v                        |
                                                     |  +-------+-------+               |
                                                     |  |   Grafana     |               |
                                                     |  | grafana.level-|               |
                                                     |  |  sony.cloud   |               |
                                                     |  +---------------+               |
                                                     +-----------------------------------+
```

## 3. Integration Prometheus — Collecte des metriques

### 3.1 Endpoints exposes par les services

Chaque service Kiddoo expose ses metriques au format OpenMetrics sur `/metrics` :

| Service        | Endpoint interne (cluster)                            | Port |
| -------------- | ----------------------------------------------------- | ---- |
| api-gateway    | `http://api-gateway.kiddoo-{env}.svc:8000/metrics`    | 8000 |
| identity-proxy | `http://identity-proxy.kiddoo-{env}.svc:8001/metrics` | 8001 |

### 3.2 ServiceMonitor (auto-discovery)

Le fichier `k8s/base/monitoring/service-monitor.yaml` configure la decouverte automatique :

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: kiddoo-services
spec:
  selector:
    matchLabels:
      app.kubernetes.io/part-of: kiddoo-platform
  endpoints:
    - port: http
      path: /metrics
      interval: 15s # Frequence de scrape
      scrapeTimeout: 10s # Timeout par scrape
  namespaceSelector:
    matchNames:
      - kiddoo-dev
      - kiddoo-test
      - kiddoo-staging
      - kiddoo-production
```

### 3.3 Verification dans Prometheus

1. Ouvrir https://prometheus.level-sony.cloud
2. Aller dans **Status > Targets**
3. Verifier que les targets `kiddoo-services` apparaissent avec l'etat `UP`
4. Tester une requete dans **Graph** :
   ```promql
   kiddoo_http_requests_total
   ```

## 4. Integration Grafana — Visualisation

### 4.1 Configurer la data source Prometheus

1. Ouvrir https://grafana.level-sony.cloud
2. Aller dans **Connections > Data Sources > Add data source**
3. Selectionner **Prometheus**
4. Configurer :

| Champ           | Valeur                                                             |
| --------------- | ------------------------------------------------------------------ |
| Name            | `Prometheus Kiddoo`                                                |
| URL             | `http://prometheus-kube-prometheus-prometheus.monitoring.svc:9090` |
| Access          | Server (default)                                                   |
| Scrape interval | `15s`                                                              |

5. Cliquer **Save & Test** — doit afficher "Data source is working"

> **Note** : Prometheus et Grafana sont dans le meme cluster Kubernetes distant.
> Grafana utilise l'URL interne du Service K8s pour communiquer avec Prometheus.
> Depuis votre navigateur local, utilisez les URLs publiques (\*.level-sony.cloud).

### 4.2 Importer le dashboard Kiddoo

**Option A — Auto-provisioning (recommande)**

Le dashboard est deploye automatiquement via le ConfigMap `kiddoo-grafana-dashboard`
dans `k8s/base/monitoring/grafana-dashboard.yaml`.
Le sidecar Grafana detecte le label `grafana_dashboard: "1"` et l'importe.

**Option B — Import manuel**

1. Aller dans **Dashboards > New > Import**
2. Copier le contenu JSON de `k8s/base/monitoring/grafana-dashboard.yaml` (champ `kiddoo-overview.json`)
3. Selectionner la data source `Prometheus Kiddoo`
4. Cliquer **Import**

### 4.3 Panels du dashboard "Kiddoo Platform Overview"

| Panel                  | Requete PromQL                                                                                                                         | Type       | SLA          |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ------------ |
| Request Rate           | `sum(rate(kiddoo_http_requests_total[5m])) by (service)`                                                                               | Timeseries | -            |
| Error Rate (%)         | `sum(rate(kiddoo_http_requests_total{status=~"5.."}[5m])) by (service) / sum(rate(kiddoo_http_requests_total[5m])) by (service) * 100` | Timeseries | < 1%         |
| Response Time p50/p99  | `histogram_quantile(0.99, sum(rate(kiddoo_http_request_duration_seconds_bucket[5m])) by (le, service))`                                | Timeseries | p99 < 1s     |
| Requests In Flight     | `kiddoo_http_requests_in_flight`                                                                                                       | Gauge      | < 100        |
| Auth Attempts          | `sum(rate(kiddoo_auth_attempts_total[5m])) by (service, result)`                                                                       | Timeseries | -            |
| Downstream Latency p99 | `histogram_quantile(0.99, sum(rate(kiddoo_downstream_request_duration_seconds_bucket[5m])) by (le, target_service))`                   | Timeseries | < 2s         |
| CPU Usage              | `sum(rate(container_cpu_usage_seconds_total{namespace=~"kiddoo-.*"}[5m])) by (pod)`                                                    | Timeseries | < 80%        |
| Memory Usage           | `sum(container_memory_working_set_bytes{namespace=~"kiddoo-.*"}) by (pod)`                                                             | Timeseries | < 85% limits |
| Pod Restarts           | `sum(increase(kube_pod_container_status_restarts_total{namespace=~"kiddoo-.*"}[1h])) by (pod)`                                         | Stat       | < 3/h        |

## 5. Indicateurs definis par categorie

### 5.1 Indicateurs de ressources (infrastructure)

| Indicateur       | Metrique Prometheus                        | Seuil alerte             | Severite |
| ---------------- | ------------------------------------------ | ------------------------ | -------- |
| CPU par pod      | `container_cpu_usage_seconds_total`        | > 80% des limits         | warning  |
| Memoire par pod  | `container_memory_working_set_bytes`       | > 85% des limits         | warning  |
| Redemarrages pod | `kube_pod_container_status_restarts_total` | > 3/heure                | warning  |
| File descriptors | `process_open_fds`                         | Built-in process metrics | info     |

### 5.2 Indicateurs de performance (SLA)

| Indicateur             | Metrique Prometheus                          | SLA Target | Seuil alerte      |
| ---------------------- | -------------------------------------------- | ---------- | ----------------- |
| Debit requetes         | `kiddoo_http_requests_total`                 | Dashboard  | -                 |
| Latence p50            | `kiddoo_http_request_duration_seconds`       | < 100ms    | > 100ms (warning) |
| Latence p99            | `kiddoo_http_request_duration_seconds`       | < 1s       | > 1s (critical)   |
| Taux d'erreur          | `kiddoo_http_requests_total{status=~"5.."}`  | < 1%       | > 1% (critical)   |
| Connexions simultanees | `kiddoo_http_requests_in_flight`             | Dashboard  | > 100 (warning)   |
| Latence downstream p99 | `kiddoo_downstream_request_duration_seconds` | < 2s       | > 2s (warning)    |
| Erreurs downstream     | `kiddoo_downstream_errors_total`             | Dashboard  | -                 |

### 5.3 Indicateurs de securite

| Indicateur               | Metrique Prometheus                                     | Seuil alerte | Severite                |
| ------------------------ | ------------------------------------------------------- | ------------ | ----------------------- |
| Taux echec auth          | `kiddoo_auth_attempts_total{result="failure"}`          | > 30%        | warning                 |
| Validations JWT echouees | `kiddoo_auth_token_validations_total{result="failure"}` | > 10/sec     | warning                 |
| Disponibilite service    | `up{job=~"kiddoo.*"}`                                   | 99.9%        | critical si down > 1min |

## 6. Alerting — PrometheusRule

### 6.1 Regles d'alerte deployees

Les alertes sont definies dans `k8s/base/monitoring/prometheus-rules.yaml` et deployees via ArgoCD.

| Alerte                        | Condition                             | Severite | Equipe   |
| ----------------------------- | ------------------------------------- | -------- | -------- |
| `ServiceDown`                 | Service indisponible > 1 min          | critical | backend  |
| `HighErrorRate`               | Taux erreur 5xx > 1% pendant 5 min    | critical | backend  |
| `HighErrorRateWarning`        | Taux erreur 5xx > 0.5% pendant 10 min | warning  | backend  |
| `HighLatencyP99`              | p99 > 1s pendant 5 min                | critical | backend  |
| `HighLatencyP50`              | p50 > 100ms pendant 10 min            | warning  | backend  |
| `HighConcurrentRequests`      | > 100 requetes en vol pendant 5 min   | warning  | backend  |
| `DownstreamServiceSlow`       | Downstream p99 > 2s pendant 5 min     | warning  | backend  |
| `HighCPUUsage`                | CPU > 80% limits pendant 10 min       | warning  | infra    |
| `HighMemoryUsage`             | Memoire > 85% limits pendant 10 min   | warning  | infra    |
| `PodRestarting`               | > 3 restarts/heure                    | warning  | backend  |
| `HighAuthFailureRate`         | > 30% echecs auth pendant 5 min       | warning  | security |
| `HighTokenValidationFailures` | > 10 echecs JWT/sec pendant 5 min     | warning  | security |

### 6.2 Verification des alertes

1. Ouvrir https://prometheus.level-sony.cloud
2. Aller dans **Alerts**
3. Verifier que les regles `kiddoo.*` apparaissent
4. Les alertes inactives sont en vert, actives en rouge

### 6.3 Configuration AlertManager (notifications)

Pour recevoir les alertes par Discord/Slack, configurer AlertManager :

```yaml
# alertmanager-config (dans le Helm values ou ConfigMap)
route:
  receiver: default
  routes:
    - match:
        severity: critical
      receiver: discord-critical
    - match:
        severity: warning
      receiver: discord-warning

receivers:
  - name: default
    webhook_configs: []
  - name: discord-critical
    webhook_configs:
      - url: "<DISCORD_WEBHOOK_URL>"
        send_resolved: true
  - name: discord-warning
    webhook_configs:
      - url: "<DISCORD_WEBHOOK_URL>"
        send_resolved: true
```

## 7. Contrats de Niveau de Service (SLA)

### 7.1 SLA par environnement

| Environnement | Disponibilite | Latence p99 | Taux d'erreur | Fenetre maintenance |
| ------------- | ------------- | ----------- | ------------- | ------------------- |
| Production    | 99.9%         | < 1s        | < 1%          | Dimanche 02h-06h    |
| Staging       | 99%           | < 2s        | < 5%          | Flexible            |
| Dev/Test      | Best effort   | -           | -             | Anytime             |

### 7.2 SLA par service

| Service        | Role                          | SLA Disponibilite | Dependances              |
| -------------- | ----------------------------- | ----------------- | ------------------------ |
| api-gateway    | Point d'entree, routage, auth | 99.9%             | identity-proxy, Keycloak |
| identity-proxy | Authentification, JWT, SSO    | 99.9%             | PostgreSQL, Keycloak     |

### 7.3 Escalade alertes

| Severite | Temps de reaction | Action                                       |
| -------- | ----------------- | -------------------------------------------- |
| critical | < 15 min          | Page on-call, investigation immediate        |
| warning  | < 1 heure         | Notification equipe, investigation planifiee |
| info     | Prochain standup  | Revue lors du daily                          |

## 8. Metriques exposees par les services

```
# Performance
kiddoo_http_requests_total{service, method, path, status}
kiddoo_http_request_duration_seconds{service, method, path}
kiddoo_http_requests_in_flight

# Securite
kiddoo_auth_attempts_total{service, result}
kiddoo_auth_token_validations_total{service, result}

# Downstream
kiddoo_downstream_request_duration_seconds{service, target_service, method, path}
kiddoo_downstream_errors_total{service, target_service, error_type}

# Process (built-in)
process_cpu_seconds_total
process_resident_memory_bytes
process_open_fds
```

## 9. Fichiers du projet

| Fichier                                      | Role                                                                  |
| -------------------------------------------- | --------------------------------------------------------------------- |
| `libs/metrics/src/lib.rs`                    | Bibliotheque Prometheus partagee (fairing Rocket + endpoint /metrics) |
| `k8s/base/monitoring/service-monitor.yaml`   | Auto-discovery des services par Prometheus                            |
| `k8s/base/monitoring/prometheus-rules.yaml`  | Regles d'alerte basees sur les SLA                                    |
| `k8s/base/monitoring/grafana-dashboard.yaml` | Dashboard Grafana auto-provisionne                                    |
| `k8s/base/api-gateway/service.yaml`          | Annotations prometheus.io pour scraping                               |
| `k8s/base/identity-proxy/service.yaml`       | Annotations prometheus.io pour scraping                               |

## 10. Checklist d'integration

- [ ] Prometheus accessible sur https://prometheus.level-sony.cloud
- [ ] Grafana accessible sur https://grafana.level-sony.cloud
- [ ] Data source Prometheus configuree dans Grafana
- [ ] ServiceMonitor deploye — targets visibles dans Prometheus > Status > Targets
- [ ] PrometheusRule deploye — alertes visibles dans Prometheus > Alerts
- [ ] Dashboard "Kiddoo Platform Overview" visible dans Grafana > Dashboards
- [ ] AlertManager configure avec notifications (Discord/Slack/Email)
- [ ] Services Kiddoo deployes et endpoint /metrics accessible
- [ ] Verifier les metriques : `kiddoo_http_requests_total` retourne des donnees
