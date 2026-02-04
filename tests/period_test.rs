// tests/period_test.rs

use dotenvy::from_path;
use spotifier_core::{Period, Result, Semester, SpotifierCoreClient};
use std::env;
use std::path::PathBuf;

/// Helper to setup authenticated client
async fn setup_client() -> Result<SpotifierCoreClient> {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    from_path(&env_path).ok();

    let nim = env::var("SPOT_NIM").expect("SPOT_NIM env var not set");
    let password = env::var("SPOT_PASSWORD").expect("SPOT_PASSWORD env var not set");

    let client = SpotifierCoreClient::new();
    client.login(&nim, &password).await?;

    Ok(client)
}

#[tokio::test]
async fn test_period_parsing() -> Result<()> {
    // Test parsing from academic year string
    let period = Period::from_academic_year_string("2025/2026 - Genap")?;
    assert_eq!(period.year, 2025);
    assert_eq!(period.semester, Semester::Even);
    assert_eq!(period.format(), "20252");

    let period2 = Period::from_academic_year_string("2024/2025 - Ganjil")?;
    assert_eq!(period2.year, 2024);
    assert_eq!(period2.semester, Semester::Odd);
    assert_eq!(period2.format(), "20241");

    let period3 = Period::from_academic_year_string("2025/2026 - SP")?;
    assert_eq!(period3.semester, Semester::Short);
    assert_eq!(period3.format(), "20253");

    // Test invalid format
    let invalid = Period::from_academic_year_string("Invalid");
    assert!(invalid.is_err());

    println!("✅ Period parsing tests passed");
    Ok(())
}

#[tokio::test]
async fn test_period_formatting() -> Result<()> {
    let period = Period::new(2025, Semester::Odd);
    assert_eq!(period.format(), "20251");
    assert_eq!(format!("{}", period), "20251");

    let period2 = Period::new(2024, Semester::Even);
    assert_eq!(period2.format(), "20242");

    println!("✅ Period formatting tests passed");
    Ok(())
}

#[tokio::test]
async fn test_change_period_valid() -> Result<()> {
    let client = setup_client().await?;

    // Get current period first
    let before = client.get_current_period_info().await?;
    println!("Current period before change: {}", before);

    // Change to a known valid period (Odd semester 2025)
    client.change_period(2025, Semester::Odd).await?;

    // Verify the change
    let after = client.get_current_period_info().await?;
    println!("Current period after change: {}", after);
    assert!(after.contains("2025/2026"));

    println!("✅ Period change test passed");
    Ok(())
}

#[tokio::test]
async fn test_get_current_period_info() -> Result<()> {
    let client = setup_client().await?;

    let period_info = client.get_current_period_info().await?;

    // Should be format like "2025/2026 - Genap" or "2025/2026 - Ganjil"
    assert!(period_info.contains("/"));
    assert!(period_info.contains(" - "));

    println!("Current period: {}", period_info);
    println!("✅ get_current_period_info test passed");

    Ok(())
}

#[tokio::test]
async fn test_semester_types() -> Result<()> {
    let client = setup_client().await?;

    // Test all semester types to ensure they don't panic
    for semester in [Semester::Odd, Semester::Even, Semester::Short] {
        let result = client.change_period(2025, semester).await;
        println!("Testing semester {:?}: {:?}", semester, result);
        // Should not panic, may succeed or fail depending on availability
    }

    println!("✅ Semester types test passed");
    Ok(())
}
