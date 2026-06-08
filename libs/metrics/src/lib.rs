/**
 * Kiddoo Platform — Prometheus Metrics Library
 *
 * Provides a shared metrics collection layer for all services.
 * Exposes HTTP request metrics, service health indicators, and
 * custom business metrics via a /metrics endpoint.
 *
 * # Indicators
 *
 * ## Resource indicators (infrastructure)
 * - `process_cpu_seconds_total` — CPU usage (built-in with `process` feature)
 * - `process_resident_memory_bytes` — Memory usage (built-in)
 * - `process_open_fds` — Open file descriptors (built-in)
 *
 * ## Performance indicators (SLA)
 * - `http_requests_total` — Total requests by method, path, status
 * - `http_request_duration_seconds` — Response time histogram
 * - `http_requests_in_flight` — Concurrent connections gauge
 *
 * ## Security indicators
 * - `auth_attempts_total` — Authentication attempts by result (success/failure)
 * - `auth_token_validations_total` — JWT token validations by result
 *
 * ## Business indicators
 * - `downstream_request_duration_seconds` — Proxy latency to downstream services
 * - `downstream_errors_total` — Errors from downstream services
 */
use once_cell::sync::Lazy;
use prometheus::{
    Encoder, Gauge, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, Response};
use std::time::Instant;

/// Global metrics registry for the service.
pub static REGISTRY: Lazy<Registry> = Lazy::new(|| {
    let registry = Registry::new_custom(Some("kiddoo".to_string()), None)
        .expect("Failed to create metrics registry");

    registry
        .register(Box::new(HTTP_REQUESTS_TOTAL.clone()))
        .expect("Failed to register http_requests_total");
    registry
        .register(Box::new(HTTP_REQUEST_DURATION.clone()))
        .expect("Failed to register http_request_duration_seconds");
    registry
        .register(Box::new(HTTP_REQUESTS_IN_FLIGHT.clone()))
        .expect("Failed to register http_requests_in_flight");
    registry
        .register(Box::new(AUTH_ATTEMPTS_TOTAL.clone()))
        .expect("Failed to register auth_attempts_total");
    registry
        .register(Box::new(AUTH_TOKEN_VALIDATIONS_TOTAL.clone()))
        .expect("Failed to register auth_token_validations_total");
    registry
        .register(Box::new(DOWNSTREAM_REQUEST_DURATION.clone()))
        .expect("Failed to register downstream_request_duration_seconds");
    registry
        .register(Box::new(DOWNSTREAM_ERRORS_TOTAL.clone()))
        .expect("Failed to register downstream_errors_total");

    registry
});

// ---------------------------------------------------------------------------
// Performance indicators (SLA)
// ---------------------------------------------------------------------------

/// Total number of HTTP requests processed, labeled by method, path, and status.
pub static HTTP_REQUESTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(
        Opts::new("http_requests_total", "Total HTTP requests processed").namespace("kiddoo"),
        &["service", "method", "path", "status"],
    )
    .expect("Failed to create http_requests_total")
});

/// Histogram of HTTP request durations in seconds.
/// Buckets are tuned for typical API SLAs: p50 < 100ms, p99 < 1s.
pub static HTTP_REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    HistogramVec::new(
        HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds",
        )
        .namespace("kiddoo")
        .buckets(vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ]),
        &["service", "method", "path"],
    )
    .expect("Failed to create http_request_duration_seconds")
});

/// Number of HTTP requests currently being processed.
pub static HTTP_REQUESTS_IN_FLIGHT: Lazy<Gauge> = Lazy::new(|| {
    Gauge::with_opts(
        Opts::new(
            "http_requests_in_flight",
            "Number of HTTP requests currently in flight",
        )
        .namespace("kiddoo"),
    )
    .expect("Failed to create http_requests_in_flight")
});

// ---------------------------------------------------------------------------
// Security indicators
// ---------------------------------------------------------------------------

/// Total authentication attempts by outcome.
pub static AUTH_ATTEMPTS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(
        Opts::new("auth_attempts_total", "Total authentication attempts").namespace("kiddoo"),
        &["service", "result"],
    )
    .expect("Failed to create auth_attempts_total")
});

/// Total JWT token validations by outcome.
pub static AUTH_TOKEN_VALIDATIONS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(
        Opts::new(
            "auth_token_validations_total",
            "Total JWT token validation attempts",
        )
        .namespace("kiddoo"),
        &["service", "result"],
    )
    .expect("Failed to create auth_token_validations_total")
});

// ---------------------------------------------------------------------------
// Downstream / proxy indicators
// ---------------------------------------------------------------------------

/// Histogram of downstream (proxied) request durations.
pub static DOWNSTREAM_REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    HistogramVec::new(
        HistogramOpts::new(
            "downstream_request_duration_seconds",
            "Downstream service request duration in seconds",
        )
        .namespace("kiddoo")
        .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        &["service", "target_service", "method", "path"],
    )
    .expect("Failed to create downstream_request_duration_seconds")
});

/// Total errors from downstream services.
pub static DOWNSTREAM_ERRORS_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    IntCounterVec::new(
        Opts::new(
            "downstream_errors_total",
            "Total errors from downstream services",
        )
        .namespace("kiddoo"),
        &["service", "target_service", "error_type"],
    )
    .expect("Failed to create downstream_errors_total")
});

// ---------------------------------------------------------------------------
// Rocket Fairing — automatic request instrumentation
// ---------------------------------------------------------------------------

/// Rocket fairing that automatically instruments all HTTP requests.
pub struct PrometheusMetrics {
    service_name: String,
}

impl PrometheusMetrics {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }
}

#[rocket::async_trait]
impl Fairing for PrometheusMetrics {
    fn info(&self) -> Info {
        Info {
            name: "Prometheus Metrics",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        HTTP_REQUESTS_IN_FLIGHT.inc();
        request.local_cache(Instant::now);
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        HTTP_REQUESTS_IN_FLIGHT.dec();

        let start = request.local_cache(Instant::now);
        let duration = start.elapsed().as_secs_f64();

        let method = request.method().as_str();
        let path = normalize_path(request.uri().path().as_str());
        let status = response.status().code.to_string();

        HTTP_REQUESTS_TOTAL
            .with_label_values(&[&self.service_name, method, &path, &status])
            .inc();

        HTTP_REQUEST_DURATION
            .with_label_values(&[&self.service_name, method, &path])
            .observe(duration);
    }
}

// ---------------------------------------------------------------------------
// Metrics endpoint handler
// ---------------------------------------------------------------------------

/// Rocket route that exposes Prometheus metrics at /metrics.
#[rocket::get("/metrics")]
pub fn metrics_endpoint() -> String {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    // Collect custom metrics
    let metric_families = REGISTRY.gather();
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");

    // Collect default process metrics
    let default_families = prometheus::gather();
    encoder
        .encode(&default_families, &mut buffer)
        .expect("Failed to encode default metrics");

    String::from_utf8(buffer).expect("Failed to convert metrics to string")
}

/// Normalizes request paths to avoid cardinality explosion.
/// Example: /api/v1/users/123 → /api/v1/users/:id
fn normalize_path(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').collect();
    let normalized: Vec<String> = segments
        .iter()
        .map(|s| {
            if s.parse::<u64>().is_ok() || is_uuid(s) {
                ":id".to_string()
            } else {
                s.to_string()
            }
        })
        .collect();
    normalized.join("/")
}

fn is_uuid(s: &str) -> bool {
    s.len() == 36 && s.chars().filter(|c| *c == '-').count() == 4
}

// ---------------------------------------------------------------------------
// Helper — measure downstream calls
// ---------------------------------------------------------------------------

/// Timer guard for measuring downstream request duration.
pub struct DownstreamTimer {
    service_name: String,
    target_service: String,
    method: String,
    path: String,
    start: Instant,
}

impl DownstreamTimer {
    pub fn start(service_name: &str, target_service: &str, method: &str, path: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            target_service: target_service.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            start: Instant::now(),
        }
    }

    pub fn finish_success(self) {
        let duration = self.start.elapsed().as_secs_f64();
        DOWNSTREAM_REQUEST_DURATION
            .with_label_values(&[
                &self.service_name,
                &self.target_service,
                &self.method,
                &self.path,
            ])
            .observe(duration);
    }

    pub fn finish_error(self, error_type: &str) {
        let duration = self.start.elapsed().as_secs_f64();
        DOWNSTREAM_REQUEST_DURATION
            .with_label_values(&[
                &self.service_name,
                &self.target_service,
                &self.method,
                &self.path,
            ])
            .observe(duration);
        DOWNSTREAM_ERRORS_TOTAL
            .with_label_values(&[&self.service_name, &self.target_service, error_type])
            .inc();
    }
}
