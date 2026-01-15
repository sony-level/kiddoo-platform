use database::pool::establish_connection_with_ssl;

#[tokio::test]
async fn test_ssl_connection() {
    let client = establish_connection_with_ssl().await;
    match client {
        Ok(_) => println!("SSL Connection successful!"),
        Err(e) => {
            // SSL may not be available on local PostgreSQL - log but don't fail
            eprintln!("SSL Connection not available: {}", e);
            // Only fail if we're in CI with a proper SSL database
            if std::env::var("CI").is_ok() {
                panic!("SSL Connection failed in CI: {}", e);
            }
        }
    }
}