// tests/integration_test.rs

use dotenvy::from_path;
use spotifier_core::{Result, SpotifierCoreClient};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// This is a complete end-to-end integration test.
/// It logs in, fetches the user profile, the course list, the details of the first course,
/// and the details of the first accessible topic in that course.
///
/// All output is written to `test_output.log` for easy inspection.
///
/// To run this test:
/// SPOT_NIM="your_nim" SPOT_PASSWORD="your_password" cargo test -- --nocapture
#[tokio::test]
async fn test_full_login_and_scrape_flow() -> Result<()> {
    // Load .env from project root
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    from_path(&env_path).ok();

    // --- SETUP: Create output directory and log file ---
    std::fs::create_dir_all("output").expect("Could not create output directory.");
    let mut log_file = File::create("output/test_output.log").expect("Could not create log file.");

    // --- SETUP: Load credentials from environment variables ---
    let nim = env::var("SPOT_NIM").expect("ERROR: SPOT_NIM environment variable not set.");
    let password =
        env::var("SPOT_PASSWORD").expect("ERROR: SPOT_PASSWORD environment variable not set.");

    writeln!(log_file, "--- Starting Full Scraper Integration Test ---").unwrap();

    let client = SpotifierCoreClient::new();

    // --- STEP 1: Login ---
    writeln!(log_file, "\n[1/5] Attempting login with NIM: {}...", nim).unwrap();
    client.login(&nim, &password).await?;
    writeln!(log_file, "[1/5] Login successful!").unwrap();

    // --- STEP 2: Fetch User Profile ---
    writeln!(log_file, "\n[2/5] Fetching user profile...").unwrap();
    let user = client.get_user_profile().await?;
    writeln!(log_file, "[2/5] Successfully fetched profile.").unwrap();
    writeln!(log_file, "{:#?}\n", user).unwrap();
    assert_eq!(user.nim, nim, "Scraped NIM must match the login NIM");

    // --- STEP 3: Fetch Course List ---
    writeln!(log_file, "\n[3/5] Fetching course list...").unwrap();
    let courses = client.get_courses().await?;
    writeln!(
        log_file,
        "[3/5] Successfully fetched {} courses.",
        courses.len()
    )
    .unwrap();
    assert!(!courses.is_empty(), "Course list should not be empty");

    // --- STEP 4 & 5: Scrape Details for the First Course and First Topic ---
    // We only test the first course to keep the integration test focused and fast.
    if let Some(first_course) = courses.first() {
        // --- STEP 4: Fetch Course Detail ---
        writeln!(
            log_file,
            "\n--------------------------------------------------"
        )
        .unwrap();
        writeln!(
            log_file,
            "[4/5] Fetching details for first course: {}...",
            first_course.name
        )
        .unwrap();
        let course_detail = client.get_course_detail(first_course).await?;
        writeln!(log_file, "[4/5] Successfully fetched course details.").unwrap();
        writeln!(log_file, "{:#?}", course_detail).unwrap();
        assert_eq!(
            course_detail.course_info.id, first_course.id,
            "Course ID mismatch"
        );

        // --- STEP 5: Fetch Topic Detail ---
        writeln!(
            log_file,
            "\n[5/5] Fetching details for the first accessible topic..."
        )
        .unwrap();
        if let Some(first_topic) = course_detail.topics.iter().find(|t| t.is_accessible) {
            let topic_detail = client.get_topic_detail(first_topic).await?;
            writeln!(log_file, "[5/5] Successfully fetched topic details.").unwrap();
            writeln!(log_file, "{:#?}", topic_detail).unwrap();

            assert_eq!(topic_detail.id, first_topic.id.unwrap());
        } else {
            writeln!(
                log_file,
                "[5/5] No accessible topics found in this course to test."
            )
            .unwrap();
        }
    } else {
        // This case should not happen if the previous assert passed, but it's good practice.
        panic!("Could not get the first course to test detail scraping.");
    }

    writeln!(
        log_file,
        "\n--- Integration Test Completed Successfully ---"
    )
    .unwrap();
    println!("Test finished. Please check 'output/test_output.log' for the detailed results.");

    Ok(())
}

/// Test the new get_course_detail_by_id method
#[tokio::test]
async fn test_get_course_detail_by_id() -> Result<()> {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    from_path(&env_path).ok();

    let nim = env::var("SPOT_NIM").expect("ERROR: SPOT_NIM environment variable not set.");
    let password =
        env::var("SPOT_PASSWORD").expect("ERROR: SPOT_PASSWORD environment variable not set.");

    let client = SpotifierCoreClient::new();
    client.login(&nim, &password).await?;

    // First get the courses to find a valid ID
    let courses = client.get_courses().await?;
    assert!(!courses.is_empty(), "Need at least one course for testing");

    let first_course_id = courses[0].id;

    // Test the ID-based method
    let course_detail = client.get_course_detail_by_id(first_course_id).await?;

    // Verify the result
    assert_eq!(
        course_detail.course_info.id, first_course_id,
        "Course ID should match"
    );
    assert!(
        !course_detail.course_info.name.is_empty(),
        "Course name should be populated"
    );

    println!(
        "✅ get_course_detail_by_id test passed for course ID: {}",
        first_course_id
    );

    Ok(())
}

/// Test the new get_topic_detail_by_id method
#[tokio::test]
async fn test_get_topic_detail_by_id() -> Result<()> {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    from_path(&env_path).ok();

    let nim = env::var("SPOT_NIM").expect("ERROR: SPOT_NIM environment variable not set.");
    let password =
        env::var("SPOT_PASSWORD").expect("ERROR: SPOT_PASSWORD environment variable not set.");

    let client = SpotifierCoreClient::new();
    client.login(&nim, &password).await?;

    // Get courses and find one with accessible topics
    let courses = client.get_courses().await?;

    for course in courses.iter() {
        let course_detail = client.get_course_detail(course).await?;

        if let Some(topic) = course_detail.topics.iter().find(|t| t.is_accessible) {
            let course_id = topic.course_id.unwrap();
            let topic_id = topic.id.unwrap();

            // Test the ID-based method
            let topic_detail = client.get_topic_detail_by_id(course_id, topic_id).await?;

            // Verify the result
            assert_eq!(topic_detail.id, topic_id, "Topic ID should match");

            println!(
                "✅ get_topic_detail_by_id test passed for course ID: {}, topic ID: {}",
                course_id, topic_id
            );

            return Ok(());
        }
    }

    println!("⚠️ No accessible topics found to test get_topic_detail_by_id");
    Ok(())
}
