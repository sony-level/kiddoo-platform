/**
 *  This module provides a database connection pool for the application.
 *  It is used to manage database connections and provide a consistent interface for database operations.
 */
use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use dotenvy::dotenv;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use std::env;
use tokio_postgres::Config;

pub type DbPool = Pool<AsyncPgConnection>;

/**
 *  Creates an async database connection pool with SSL support.
 * @return {DbPool} - A connection pool to the database.
 *
 */
pub async fn create_pool() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
    Pool::builder(config)
        .build()
        .expect("Failed to create pool")
}

/**
 *  Establishes a single async connection to the database with SSL.
 * @return {AsyncPgConnection} - A connection to the database.
 *
 */
pub async fn establish_connection_with_ssl()
-> Result<tokio_postgres::Client, Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Control acceptance of invalid TLS certificates via environment variables.
    // DB_ACCEPT_INVALID_CERTS: "true" or "false" (default: "false").
    // APP_ENV: "production" or other (default: "development").
    // let accept_invalid_certs = env::var("DB_ACCEPT_INVALID_CERTS")
    //     .unwrap_or_else(|_| "false".to_string())
    //     .eq_ignore_ascii_case("true");
    // let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    // if app_env.eq_ignore_ascii_case("production") {
    //     panic!(
    //         "Invalid configuration: DB_ACCEPT_INVALID_CERTS must not be enabled in production."
    //     );
    // }

    let builder = TlsConnector::builder();
    // if accept_invalid_certs {
    // Allow invalid/self-signed certificates only in non-production environments.
    //  builder.danger_accept_invalid_certs(true);
    // }

    let tls_connector = builder.build()?;
    let connector = MakeTlsConnector::new(tls_connector);

    let config: Config = database_url.parse()?;
    let (client, connection) = config.connect(connector).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    Ok(client)
}

/**
 *  This function establishes a synchronous connection to the database (no SSL).
 * @establish_connection {PgConnection} - A connection to the database.
 */
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
