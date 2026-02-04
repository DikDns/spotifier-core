// tests/cache_test.rs

use dotenvy::from_path;
use spotifier_core::{DelayConfig, FileCache, Result, SpotifierCoreClient};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[tokio::test]
async fn test_cookie_persistence() -> Result<()> {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    from_path(&env_path).ok();

    let nim = env::var("SPOT_NIM").expect("SPOT_NIM env var not set");
    let password = env::var("SPOT_PASSWORD").expect("SPOT_PASSWORD env var not set");

    let cookie_path = Path::new("test_cookies.json");
    if cookie_path.exists() {
        std::fs::remove_file(cookie_path).ok();
    }

    {
        println!("Stage 1: Login and save cookies...");
        let client = SpotifierCoreClient::new();
        client.login(&nim, &password).await?;
        client.save_cookies(cookie_path).await?;
        println!("✅ Cookies saved to {:?}", cookie_path);
    }

    {
        println!("Stage 2: Create new client and load cookies...");
        let client = SpotifierCoreClient::new();
        client.load_cookies(cookie_path).await?;

        println!("Fetching profile using loaded cookies...");
        let user = client.get_user_profile().await?;
        println!("✅ Logged in as: {}", user.name);
        assert!(!user.name.is_empty());
    }

    std::fs::remove_file(cookie_path).ok();
    Ok(())
}

#[tokio::test]
async fn test_course_list_caching() -> Result<()> {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    from_path(&env_path).ok();

    let nim = env::var("SPOT_NIM").expect("SPOT_NIM env var not set");
    let password = env::var("SPOT_PASSWORD").expect("SPOT_PASSWORD env var not set");

    let cache_dir = Path::new("test_cache");
    if cache_dir.exists() {
        std::fs::remove_dir_all(cache_dir).ok();
    }

    let mut client = SpotifierCoreClient::with_config(DelayConfig {
        enabled: false, // Disable delays for faster test
        ..Default::default()
    });

    let cache = Arc::new(FileCache::new(cache_dir));
    client.set_cache(cache);

    client.login(&nim, &password).await?;

    println!("Fetching courses for the first time (caching)...");
    let courses1 = client.get_courses().await?;
    assert!(!courses1.is_empty());

    let cache_file = cache_dir.join("courses.json");
    assert!(cache_file.exists(), "Cache file should be created");

    println!("Fetching courses for the second time (from cache)...");
    // We can simulate offline or just check if it works
    let courses2 = client.get_courses().await?;
    assert_eq!(courses1.len(), courses2.len());
    println!("✅ Cache works! Fetched {} courses.", courses2.len());

    std::fs::remove_dir_all(cache_dir).ok();
    Ok(())
}
