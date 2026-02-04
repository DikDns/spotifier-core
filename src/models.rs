use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub name: String,
    pub nim: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Course {
    pub id: u64,
    pub code: String,
    pub name: String,
    pub credits: u8,
    pub lecturer: String,
    pub academic_year: String,
    pub href: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rps {
    pub id: Option<u64>,
    pub href: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicInfo {
    pub id: Option<u64>,
    pub course_id: Option<u64>,
    pub access_time: Option<NaiveDateTime>,
    pub is_accessible: bool,
    pub href: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetailCourse {
    #[serde(flatten)]
    pub course_info: Course,
    pub description: String,
    pub rps: Rps,
    pub topics: Vec<TopicInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub id: u32,
    pub youtube_id: Option<String>,
    pub raw_html: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TaskStatus {
    Pending,
    Submitted,
    Graded,
    NotSubmitted,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Answer {
    pub id: Option<u64>,
    pub content: String,
    pub file_href: Option<String>,
    pub is_graded: bool,
    pub lecturer_notes: String,
    pub score: f32,
    pub date_submitted: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: Option<u64>,
    pub course_id: u64,
    pub topic_id: u64,
    pub token: String,
    pub title: String,
    pub description: String,
    pub file: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub due_date: Option<NaiveDateTime>,
    pub status: TaskStatus,
    pub answer: Option<Answer>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicDetail {
    pub id: u64,
    pub access_time: Option<NaiveDateTime>,
    pub is_accessible: bool,
    pub href: String,
    pub description: Option<String>,
    pub contents: Vec<Content>,
    pub tasks: Vec<Task>,
}

/// Represents the type of academic semester
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Semester {
    /// Odd semester (Ganjil) - typically Sep-Jan
    Odd = 1,
    /// Even semester (Genap) - typically Feb-June
    Even = 2,
    /// Short semester (SP/Semester Pendek) - typically July-Aug
    Short = 3,
}

impl Semester {
    /// Convert to numeric value for period format
    pub fn as_num(&self) -> u8 {
        *self as u8
    }
}

/// Helper struct to format and parse academic periods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    pub year: u16,
    pub semester: Semester,
}

impl Period {
    pub fn new(year: u16, semester: Semester) -> Self {
        Self { year, semester }
    }

    /// Format as YYYYN string (e.g., "20251")
    pub fn format(&self) -> String {
        format!("{}{}", self.year, self.semester.as_num())
    }

    /// Parse academic year string to Period.
    ///
    /// Accepts formats like:
    /// - "2025/2026 - Genap" -> Period { year: 2025, semester: Semester::Even }
    /// - "2025/2026 - Ganjil" -> Period { year: 2025, semester: Semester::Odd }
    /// - "2024/2025 - SP" -> Period { year: 2024, semester: Semester::Short }
    pub fn from_academic_year_string(s: &str) -> crate::error::Result<Self> {
        use crate::error::ScraperError;

        // Parse "2025/2026 - Genap" format
        let parts: Vec<&str> = s.split('-').map(|p| p.trim()).collect();

        if parts.len() != 2 {
            return Err(ScraperError::ParsingError(format!(
                "Invalid academic year format: {}",
                s
            )));
        }

        // Parse year from "2025/2026" -> 2025
        let year_part = parts[0];
        let year = year_part
            .split('/')
            .next()
            .and_then(|y| y.parse::<u16>().ok())
            .ok_or_else(|| {
                ScraperError::ParsingError(format!("Cannot parse year from: {}", year_part))
            })?;

        // Parse semester: "Genap", "Ganjil", "SP"
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

/// Configuration for human-like delays between requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayConfig {
    /// Minimum delay in milliseconds
    pub min_delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Whether delays are enabled
    pub enabled: bool,
}

impl Default for DelayConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 1000,
            max_delay_ms: 3000,
            enabled: true,
        }
    }
}
