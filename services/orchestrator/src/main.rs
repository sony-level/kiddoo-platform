use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    println!("🚀 Starting Kidoo services...");

    // Start identity-proxy first (api-gateway depends on it)
    let mut identity_proxy = start_service("identity-proxy", 8001);

    // Wait a bit for identity-proxy to start
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Start api-gateway
    let mut api_gateway = start_service("api-gateway", 8000);

    println!("✅ All services started!");
    println!("   - API Gateway: http://127.0.0.1:8000");
    println!("   - Identity Proxy: http://127.0.0.1:8001");
    println!("   - Swagger UI: http://127.0.0.1:8000/swagger-ui/");

    // Wait for Ctrl+C
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\n Shutting down services...");
    let _ = api_gateway.kill();
    let _ = api_gateway.wait();
    let _ = identity_proxy.kill();
    let _ = identity_proxy.wait();
    println!(" Goodbye!");
}

fn start_service(name: &str, port: u16) -> Child {
    Command::new("cargo")
        .args(["run", "-p", name])
        .env("ROCKET_PORT", port.to_string())
        .env("ROCKET_ADDRESS", "127.0.0.1")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to start {}", name))
}
