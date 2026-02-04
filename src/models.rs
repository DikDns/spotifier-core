use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Represents a user profile on the SPOT platform.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    /// The full name of the student.
    pub name: String,
    /// The unique student identification number (NIM).
    pub nim: String,
}

/// Represents a course the student is currently or was previously enrolled in.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Course {
    /// Unique internal identifier for the course.
    pub id: u64,
    /// The official course code (e.g., "IK410").
    pub code: String,
    /// The full name of the course.
    pub name: String,
    /// Number of academic credits (SKS) awarded for the course.
    pub credits: u8,
    /// The name of the primary lecturer for the course.
    pub lecturer: String,
    /// The academic year and semester string (e.g., "2025/2026 - Ganjil").
    pub academic_year: String,
    /// The URL path to the course's detail page.
    pub href: String,
}

/// Information about the Rencana Pembelajaran Semester (RPS/Syllabus).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rps {
    /// Unique identifier for the RPS document.
    pub id: Option<u64>,
    /// URL path to download the RPS file.
    pub href: Option<String>,
}

/// Summary information for a learning topic within a course.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicInfo {
    /// Unique identifier for the topic.
    pub id: Option<u64>,
    /// The ID of the course this topic belongs to.
    pub course_id: Option<u64>,
    /// The timestamp when the user last accessed this topic.
    pub access_time: Option<NaiveDateTime>,
    /// Whether the topic is currently open for students.
    pub is_accessible: bool,
    /// URL path to the topic's detail page.
    pub href: Option<String>,
}

/// Detailed data for a specific course, including its list of topics.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetailCourse {
    /// Basic identification and meta-data for the course.
    #[serde(flatten)]
    pub course_info: Course,
    /// A textual description or overview of the course.
    pub description: String,
    /// Syllabus information.
    pub rps: Rps,
    /// List of learning topics available in this course.
    pub topics: Vec<TopicInfo>,
}

/// A specific content item (e.g., video or reading material) within a topic.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    /// Unique identifier for the content piece.
    pub id: u32,
    /// The YouTube video ID, if the content is an embedded video.
    pub youtube_id: Option<String>,
    /// Raw HTML content for text/article-based materials.
    pub raw_html: String,
}

/// The current status of a student's task submission.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TaskStatus {
    /// Task has not been interacted with yet.
    Pending,
    /// Task has been successfully uploaded but not yet graded.
    Submitted,
    /// Task has been reviewed and assigned a score.
    Graded,
    /// The submission deadline has passed without a submission.
    NotSubmitted,
}

/// Represents a student's submission for a specific task.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Answer {
    /// Unique identifier for the submission.
    pub id: Option<u64>,
    /// The text content or description provided by the student.
    pub content: String,
    /// URL to the uploaded file, if any.
    pub file_href: Option<String>,
    /// Whether the lecturer has graded this submission.
    pub is_graded: bool,
    /// Feedback or notes provided by the lecturer.
    pub lecturer_notes: String,
    /// The numerical score awarded (0.0 - 100.0).
    pub score: f32,
    /// The timestamp when the submission was uploaded.
    pub date_submitted: Option<NaiveDateTime>,
}

/// Represents a task or assignment within a topic.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    /// Unique identifier for the task.
    pub id: Option<u64>,
    /// The ID of the parent course.
    pub course_id: u64,
    /// The ID of the parent topic.
    pub topic_id: u64,
    /// Security token required for submission.
    pub token: String,
    /// The title of the assignment.
    pub title: String,
    /// Detailed instructions for the task.
    pub description: String,
    /// Name of the reference file provided by the lecturer, if any.
    pub file: Option<String>,
    /// When the task becomes available for submission.
    pub start_date: Option<NaiveDateTime>,
    /// The final deadline for submission.
    pub due_date: Option<NaiveDateTime>,
    /// The current submission status for the student.
    pub status: TaskStatus,
    /// The student's submission data, if they have uploaded anything.
    pub answer: Option<Answer>,
}

/// Full details of a topic, including all instructional content and assignments.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicDetail {
    /// Unique identifier for the topic.
    pub id: u64,
    /// When the topic was last accessed by the user.
    pub access_time: Option<NaiveDateTime>,
    /// Whether students can currently view this topic.
    pub is_accessible: bool,
    /// The direct URL path to the topic page.
    pub href: String,
    /// General description of the topic.
    pub description: Option<String>,
    /// List of instructional materials (text, video).
    pub contents: Vec<Content>,
    /// List of assignments/tasks associated with this topic.
    pub tasks: Vec<Task>,
}

/// Represents the type of academic semester.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Semester {
    /// Odd semester (Ganjil) - typically September to January.
    Odd = 1,
    /// Even semester (Genap) - typically February to June.
    Even = 2,
    /// Short semester (Semester Pendek) - typically July to August.
    Short = 3,
}

impl Semester {
    /// Converts the semester type to its numeric equivalent used in the SPOT URL format.
    pub fn as_num(&self) -> u8 {
        *self as u8
    }
}

/// A helper for representing and formatting academic periods (Year + Semester).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    /// The starting year of the academic period (e.g., 2025).
    pub year: u16,
    /// The specific semester type.
    pub semester: Semester,
}

impl Period {
    /// Creates a new `Period` instance.
    pub fn new(year: u16, semester: Semester) -> Self {
        Self { year, semester }
    }

    /// Formats the period into the "YYYYN" format expected by the SPOT URL scheme.
    ///
    /// Example: `Period { year: 2025, semester: Semester::Odd }.format()` returns `"20251"`.
    pub fn format(&self) -> String {
        format!("{}{}", self.year, self.semester.as_num())
    }

    /// Parses a human-readable academic year string into a `Period`.
    ///
    /// # Expected Formats:
    /// - `"2025/2026 - Genap"` -> `Period { year: 2025, semester: Semester::Even }`
    /// - `"2025/2026 - Ganjil"` -> `Period { year: 2025, semester: Semester::Odd }`
    /// - `"2024/2025 - SP"` -> `Period { year: 2024, semester: Semester::Short }`
    pub fn from_academic_year_string(s: &str) -> crate::error::Result<Self> {
        use crate::error::ScraperError;

        let parts: Vec<&str> = s.split('-').map(|p| p.trim()).collect();

        if parts.len() != 2 {
            return Err(ScraperError::ParsingError(format!(
                "Invalid academic year format: {}",
                s
            )));
        }

        // Parse starting year from "2025/2026"
        let year_part = parts[0];
        let year = year_part
            .split('/')
            .next()
            .and_then(|y| y.parse::<u16>().ok())
            .ok_or_else(|| {
                ScraperError::ParsingError(format!("Cannot parse year from: {}", year_part))
            })?;

        // Parse semester: "Genap" (Even), "Ganjil" (Odd), "SP" (Short)
        let semester = match parts[1].to_lowercase().as_str() {
            "genap" => Semester::Even,
            "ganjil" => Semester::Odd,
            "sp" | "semester pendek" => Semester::Short,
            other => {
                return Err(ScraperError::ParsingError(format!(
                    "Unknown semester type: {}",
                    other
                )));
            }
        };

        Ok(Period::new(year, semester))
    }
}

impl std::fmt::Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Configuration for simulating human browsing behavior via randomized delays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayConfig {
    /// Minimum sleep duration in milliseconds between requests.
    pub min_delay_ms: u64,
    /// Maximum sleep duration in milliseconds between requests.
    pub max_delay_ms: u64,
    /// Whether the randomized delay logic is active.
    pub enabled: bool,
}

impl Default for DelayConfig {
    /// Default configuration: 1000ms - 3000ms, enabled.
    fn default() -> Self {
        Self {
            min_delay_ms: 1000,
            max_delay_ms: 3000,
            enabled: true,
        }
    }
}
