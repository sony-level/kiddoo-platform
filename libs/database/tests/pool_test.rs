use database::pool::establish_connection_with_ssl;

#[tokio::test]
async fn test_ssl_connection() {
    let client = establish_connection_with_ssl().await;
    match client {
        Ok(_) => println!("SSL Connection successful!"),
        Err(e) => panic!("SSL Connection failed: {}", e),
    }
}
