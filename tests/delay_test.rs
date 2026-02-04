// tests/delay_test.rs

use dotenvy::from_path;
use spotifier_core::{DelayConfig, Result, SpotifierCoreClient};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

#[tokio::test]
async fn test_human_like_delays() -> Result<()> {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    from_path(&env_path).ok();

    let nim = env::var("SPOT_NIM").expect("SPOT_NIM env var not set");
    let password = env::var("SPOT_PASSWORD").expect("SPOT_PASSWORD env var not set");

    // Use a very short delay for testing to not waste too much time,
    // but long enough to measure
    let config = DelayConfig {
        min_delay_ms: 500,
        max_delay_ms: 1000,
        enabled: true,
    };

    let client = SpotifierCoreClient::with_config(config);

    let start = Instant::now();
    println!("Logging in (expecting 2-5s jitter + base delay)...");
    client.login(&nim, &password).await?;
    let login_duration = start.elapsed();
    println!("Login took {:?}", login_duration);

    // Login has 2-5s jitter + 2 requests (SSO GET + SSO POST)
    // Min time: 2s (jitter) + 2 * 0.5s (requests) = 3s
    assert!(
        login_duration.as_millis() >= 2500,
        "Login should take at least 2.5s with delays enabled"
    );

    let start = Instant::now();
    println!("Fetching courses (expecting 0.5-1s delay)...");
    client.get_courses().await?;
    let courses_duration = start.elapsed();
    println!("Fetching courses took {:?}", courses_duration);

    assert!(
        courses_duration.as_millis() >= 450,
        "Request should take at least 450ms with 500ms min delay"
    );

    Ok(())
}
