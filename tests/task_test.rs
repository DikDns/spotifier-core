// tests/task_test.rs

use dotenvy::from_path;
use spotifier_core::{Result, Semester, SpotifierCoreClient, TaskStatus};
use std::env;
use std::path::PathBuf;

#[tokio::test]
async fn test_task_lifecycle() -> Result<()> {
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

    println!("Topic: {}", topic.href);
    println!("Number of tasks found: {}", topic.tasks.len());

    if topic.tasks.is_empty() {
        println!("Topic description: {:?}", topic.description);
        println!("Contents count: {}", topic.contents.len());

        // If no tasks in this specific topic, let's see what's in the course
        println!("Fetching course detail for context...");
        let course = client.get_course_detail_by_id(course_id).await?;
        println!("Course: {}", course.course_info.name);
        println!("Available topics in this course:");
        for (i, t) in course.topics.iter().enumerate() {
            println!("  [{}]: ID={:?}, Title= (check model)", i, t.id);
        }
    }

    // Verify tasks exist
    assert!(!topic.tasks.is_empty(), "No tasks found in this topic");
    let task = &topic.tasks[0];

    let task_id = task.id.expect("Task ID should be present");
    let token = &task.token;
    assert!(!token.is_empty(), "CSRF token should be present");

    println!("Found task: {} (Status: {:?})", task.title, task.status);

    // If already submitted, delete it first to ensure clean test
    if let Some(answer) = &task.answer {
        if let Some(answer_id) = answer.id {
            println!("Cleaning up existing submission with ID {}...", answer_id);
            client
                .delete_task_submission(course_id, topic_id, answer_id)
                .await?;
        }
    }

    // Step 3: Submit Task (Create)
    let test_content = format!("Automated Test Submission at {}", chrono::Local::now());
    println!("Submitting task with content: \"{}\"...", test_content);

    // Test with text only
    client
        .submit_task(
            course_id,
            topic_id,
            task_id,
            token,
            &test_content,
            None,
            None,
        )
        .await?;
    println!("✅ Task submitted successfully");

    // Re-fetch to verify status
    let updated_topic = client.get_topic_detail_by_id(course_id, topic_id).await?;
    let updated_task = &updated_topic.tasks[0];
    assert!(
        matches!(updated_task.status, TaskStatus::Submitted),
        "Task status should be Submitted"
    );

    let answer = updated_task
        .answer
        .as_ref()
        .expect("Answer should be present after submission");
    let answer_id = answer
        .id
        .expect("Answer ID should be present after submission");
    assert_eq!(
        answer.content.trim(),
        test_content.trim(),
        "Submission content mismatch"
    );
    println!("✅ Task status verified: Submitted");

    // Step 4: Delete Task (Delete)
    println!("Deleting task submission with ID {}...", answer_id);
    client
        .delete_task_submission(course_id, topic_id, answer_id)
        .await?;
    println!("✅ Task deletion successful");

    // Re-fetch to verify deletion
    let final_topic = client.get_topic_detail_by_id(course_id, topic_id).await?;
    let final_task = &final_topic.tasks[0];
    assert!(
        matches!(final_task.status, TaskStatus::NotSubmitted),
        "Task status should be NotSubmitted after deletion"
    );
    assert!(
        final_task.answer.is_none(),
        "Answer should be None after deletion"
    );
    println!("✅ Task deletion verified: Status is NotSubmitted");

    Ok(())
}
