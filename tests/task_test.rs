// tests/task_test.rs

use dotenvy::from_path;
use spotifier_core::{Result, Semester, SpotifierCoreClient, TaskStatus};
use std::env;
use std::path::PathBuf;

#[tokio::test]
async fn test_task_submission_with_file() -> Result<()> {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    from_path(&env_path).ok();

    let nim = env::var("SPOT_NIM").expect("SPOT_NIM env var not set");
    let password = env::var("SPOT_PASSWORD").expect("SPOT_PASSWORD env var not set");

    let client = SpotifierCoreClient::new();
    client.login(&nim, &password).await?;

    // Step 1: Switch to 2025 Odd
    println!("Switching to 2025 Odd semester...");
    client.change_period(2025, Semester::Odd).await?;

    let current_period = client.get_current_period_info().await?;
    println!("Current active period: {}", current_period);

    // Step 2: Get Topic Detail (Read)
    let course_id = 2510009532;
    let topic_id = 1358801;
    println!(
        "Fetching topic detail for course {} topic {}...",
        course_id, topic_id
    );
    let topic = client.get_topic_detail_by_id(course_id, topic_id).await?;

    // Verify tasks exist
    assert!(!topic.tasks.is_empty(), "No tasks found in this topic");
    let task = &topic.tasks[0];

    let task_id = task.id.expect("Task ID should be present");
    let token = &task.token;
    assert!(!token.is_empty(), "CSRF token should be present");

    println!("DEBUG: Task ID={}, Token Len={}", task_id, token.len());
    println!("Found task: {} (Status: {:?})", task.title, task.status);

    // If already submitted, delete it first to ensure clean test
    if let Some(answer) = &task.answer {
        if let Some(answer_id) = answer.id {
            println!("Cleaning up existing submission with ID {}...", answer_id);
            client
                .delete_task_submission(course_id, topic_id, answer_id)
                .await?;
            // Wait a bit after deletion
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    // Step 3: Submit Task (Create) with File
    let test_content = format!(
        "Automated Test Submission (with file) at {}",
        chrono::Local::now()
    );
    let mut file_name = "test_upload.pdf".to_string();
    let mut file_data =
        b"%PDF-1.4\n1 0 obj\n<< /Title (Test) >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF"
            .to_vec();

    // Try to load local file if exists
    let local_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("test-file.pdf");
    if local_file_path.exists() {
        println!(
            "DEBUG: Local test file found at {:?}, using it.",
            local_file_path
        );
        if let Ok(data) = std::fs::read(&local_file_path) {
            file_data = data;
            file_name = local_file_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
        }
    } else {
        println!(
            "DEBUG: Local test file not found at {:?}, using dummy data.",
            local_file_path
        );
    }

    println!(
        "Submitting task with content: \"{}\" and file: {}...",
        test_content, file_name
    );

    client
        .submit_task(
            course_id,
            topic_id,
            task_id,
            token,
            &test_content,
            Some(file_name),
            Some(file_data),
        )
        .await?;
    println!("✅ Task submitted successfully. Waiting 3s for server to sync...");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Re-fetch to verify status
    let updated_topic = client.get_topic_detail_by_id(course_id, topic_id).await?;
    let updated_task = &updated_topic.tasks[0];

    if !matches!(updated_task.status, TaskStatus::Submitted) {
        println!("DEBUG: Task detail after re-fetch: {:#?}", updated_task);
    }

    assert!(
        matches!(updated_task.status, TaskStatus::Submitted),
        "Task status should be Submitted after re-fetch. Current: {:?}",
        updated_task.status
    );

    let answer = updated_task
        .answer
        .as_ref()
        .expect("Answer should be present after submission");

    assert!(
        answer.file_href.is_some(),
        "File href should be present after upload"
    );
    println!("✅ Task status verified: Submitted with file");
    println!(
        "Check the submission manually at: https://spot.upi.edu/mhs/topik/{}/{}",
        course_id, topic_id
    );

    Ok(())
}
