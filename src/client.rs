use crate::error::{Result, ScraperError};
use crate::models::{
    Course, DelayConfig, DetailCourse, Period, Semester, TopicDetail, TopicInfo, User,
};
use crate::parsers;
use rand::Rng;
use reqwest::cookie::Jar;
use reqwest::header::{HeaderMap, USER_AGENT};
use reqwest::multipart;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::sleep;

const SSO_LOGIN_PAGE_URL: &str =
    "https://sso.upi.edu/cas/login?service=https://spot.upi.edu/beranda";

const MODERN_USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Safari/605.1.15",
];

pub struct SpotifierCoreClient {
    client: reqwest::Client,
    base_url: String,
    delay_config: DelayConfig,
}

impl SpotifierCoreClient {
    pub fn new() -> Self {
        Self::with_config(DelayConfig::default())
    }

    pub fn with_config(delay_config: DelayConfig) -> Self {
        let cookie_jar = Arc::new(Jar::default());

        let mut headers = HeaderMap::new();
        // Pick a random User-Agent at initialization
        let ua = MODERN_USER_AGENTS[rand::rng().random_range(0..MODERN_USER_AGENTS.len())];
        headers.insert(USER_AGENT, ua.parse().unwrap());

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .cookie_provider(cookie_jar)
            .default_headers(headers) // Use the headers
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://spot.upi.edu".to_string(),
            delay_config,
        }
    }

    /// Sets a new delay configuration for the client.
    pub fn set_delay_config(&mut self, config: DelayConfig) {
        self.delay_config = config;
    }

    fn get_random_ua(&self) -> &'static str {
        MODERN_USER_AGENTS[rand::rng().random_range(0..MODERN_USER_AGENTS.len())]
    }

    /// Waits for a random duration based on delay_config.
    async fn wait_random(&self) {
        if !self.delay_config.enabled {
            return;
        }

        let ms = rand::rng()
            .random_range(self.delay_config.min_delay_ms..=self.delay_config.max_delay_ms);
        sleep(std::time::Duration::from_millis(ms)).await;
    }

    /// Helper to perform a GET request with randomized delay and rotated UA.
    async fn get_request(&self, url: &str) -> Result<reqwest::Response> {
        self.wait_random().await;
        let ua = self.get_random_ua();
        self.client
            .get(url)
            .header(USER_AGENT, ua)
            .send()
            .await
            .map_err(ScraperError::from)
    }

    /// Helper to perform a POST request with randomized delay and rotated UA.
    async fn post_request<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        form: &T,
    ) -> Result<reqwest::Response> {
        self.wait_random().await;
        let ua = self.get_random_ua();
        self.client
            .post(url)
            .header(USER_AGENT, ua)
            .form(form)
            .send()
            .await
            .map_err(ScraperError::from)
    }

    /// Logs into SPOT using a student ID (NIM) and password.
    pub async fn login(&self, nim: &str, password: &str) -> Result<()> {
        // --- STEP 1: GET the login page to get the "execution" token ---
        let response = self.get_request(SSO_LOGIN_PAGE_URL).await?;

        // The service URL is now part of the request URL itself
        let login_action_url = response.url().clone();

        let response_text = response.text().await?;
        let document = Html::parse_document(&response_text);

        let token_selector = Selector::parse("input[name=\"execution\"]").unwrap();

        let execution_token = document
            .select(&token_selector)
            .next()
            .and_then(|element| element.value().attr("value"))
            .ok_or(ScraperError::TokenNotFound)?;

        // --- STEP 2: POST credentials to the correct URL with all fields ---
        let mut params = HashMap::new();
        params.insert("username", nim);
        params.insert("password", password);
        params.insert("execution", execution_token);
        params.insert("_eventId", "submit");

        // --- CHANGE 2: Post to the full URL including the '?service=...' part ---
        let response = self
            .post_request(login_action_url.as_str(), &params)
            .await?;

        // Action-specific jitter: Add a longer delay (2-5 seconds) after successful login.
        if self.delay_config.enabled {
            let jitter_ms = rand::rng().random_range(2000..=5000);
            println!("DEBUG: Login success jitter ({}ms)...", jitter_ms);
            sleep(std::time::Duration::from_millis(jitter_ms)).await;
        }

        // --- STEP 3: Verify the final redirection URL ---
        let final_url = response.url().clone();
        if final_url.host_str() != Some("spot.upi.edu") {
            let error_body = response.text().await.unwrap_or_default();
            std::fs::write("login_fail.html", error_body).ok();
            println!(
                "Login failed. Check login_fail.html for details. The final URL was: {}",
                final_url
            );

            return Err(ScraperError::AuthenticationFailed);
        }

        Ok(())
    }

    async fn get_html(&self, path: &str) -> Result<String> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.get_request(&url).await?;

        if !response.url().path().starts_with(path) {
            return Err(ScraperError::SessionExpired);
        }

        Ok(response.text().await?)
    }

    pub async fn get_user_profile(&self) -> Result<User> {
        let html_content = self.get_html("/mhs").await?;
        parsers::user::parse_user_from_html(&html_content)
    }

    pub async fn get_courses(&self) -> Result<Vec<Course>> {
        let html_content = self.get_html("/mhs").await?;
        parsers::courses::parse_courses_from_html(&html_content)
    }

    pub async fn get_course_detail(&self, course: &Course) -> Result<DetailCourse> {
        // The href for the detail page is already stored in the Course struct
        let html_content: String = self.get_html(&course.href).await?;
        parsers::course_detail::parse_course_detail_from_html(&html_content, course.clone())
    }

    pub async fn get_topic_detail(&self, topic_info: &TopicInfo) -> Result<TopicDetail> {
        let href = topic_info.href.as_ref().ok_or_else(|| {
            ScraperError::ParsingError("TopicInfo tidak memiliki href yang valid".to_string())
        })?;

        let course_id = topic_info.course_id.ok_or_else(|| {
            ScraperError::ParsingError("TopicInfo tidak memiliki course_id yang valid".to_string())
        })?;
        let topic_id = topic_info.id.ok_or_else(|| {
            ScraperError::ParsingError("TopicInfo tidak memiliki id yang valid".to_string())
        })?;

        let html_content = self.get_html(href).await?;
        parsers::topic_detail::parse_topic_detail_from_html(&html_content, topic_id, course_id)
    }

    pub async fn get_course_detail_by_id(&self, course_id: u64) -> Result<DetailCourse> {
        let courses = self.get_courses().await?;

        let course = courses
            .into_iter()
            .find(|c| c.id == course_id)
            .ok_or_else(|| {
                ScraperError::ParsingError(format!("Course with ID {} not found", course_id))
            })?;

        self.get_course_detail(&course).await
    }

    pub async fn get_topic_detail_by_id(
        &self,
        course_id: u64,
        topic_id: u64,
    ) -> Result<TopicDetail> {
        let path = format!("/mhs/topik/{}/{}", course_id, topic_id);
        let html_content = self.get_html(&path).await?;
        parsers::topic_detail::parse_topic_detail_from_html(&html_content, topic_id, course_id)
    }

    pub async fn change_period(&self, year: u16, semester: Semester) -> Result<()> {
        let period = Period::new(year, semester);
        let path = format!("/adm/semester/{}", period.format());

        let response = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await?;

        let status = response.status();

        // Success: 200 OK or 302 redirect to /adm
        if status.is_success() || status.is_redirection() {
            // Check if we actually redirected to /adm (success indicator)
            let final_url = response.url();
            if final_url.path() == "/adm" || final_url.path().starts_with("/adm") {
                return Ok(());
            }
        }

        // 500 error means invalid period format
        if status.as_u16() == 500 {
            return Err(ScraperError::InvalidPeriod(format!(
                "Period {} is not valid or unavailable",
                period.format()
            )));
        }

        // Any other error
        Err(ScraperError::ParsingError(format!(
            "Failed to change period, status: {}",
            status
        )))
    }

    pub async fn get_current_period_info(&self) -> Result<String> {
        let courses = self.get_courses().await?;

        if let Some(first_course) = courses.first() {
            // Return raw string: "2025/2026 - Genap" or "2025/2026 - Ganjil"
            Ok(first_course.academic_year.clone())
        } else {
            Err(ScraperError::ParsingError(
                "No courses found to determine current period".to_string(),
            ))
        }
    }

    /// Submits a task to SPOT.
    ///
    /// # Arguments
    /// * `course_id` - ID of the course
    /// * `topic_id` - ID of the topic
    /// * `task_id` - ID of the task
    /// * `token` - CSRF token for submission (from Task struct)
    /// * `content` - Text content/description of the submission
    /// * `file_name` - Optional filename for attachment
    /// * `file_data` - Optional file content as bytes
    pub async fn submit_task(
        &self,
        course_id: u64,
        topic_id: u64,
        task_id: u64,
        token: &str,
        content: &str,
        file_name: Option<String>,
        file_data: Option<Vec<u8>>,
    ) -> Result<()> {
        let mut form = multipart::Form::new()
            .text("_token", token.to_string())
            .text("id_pn", course_id.to_string())
            .text("id_pt", topic_id.to_string())
            .text("id_tg", task_id.to_string())
            .text("isi", content.to_string());

        if let (Some(name), Some(data)) = (file_name, file_data) {
            let part = multipart::Part::bytes(data).file_name(name);
            form = form.part("filename", part);
        }

        let response = self
            .client
            .post(format!("{}/mhs/tugas_store", self.base_url))
            .multipart(form)
            .send()
            .await?;

        let status = response.status();

        // Success usually redirects back to the topic page (302 -> 200)
        if status.is_success() || status.is_redirection() {
            return Ok(());
        }

        Err(ScraperError::TaskSubmissionFailed(format!(
            "Status: {}",
            status
        )))
    }

    /// Deletes a task submission from SPOT.
    ///
    /// # Arguments
    /// * `course_id` - ID of the course
    /// * `topic_id` - ID of the topic
    /// * `answer_id` - ID of the submission to delete (from Answer struct)
    pub async fn delete_task_submission(
        &self,
        course_id: u64,
        topic_id: u64,
        answer_id: u64,
    ) -> Result<()> {
        let path = format!("/mhs/tugas_del/{}/{}/{}", course_id, topic_id, answer_id);
        let response = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await?;

        let status = response.status();

        // Success usually redirects back to the topic page (302 -> 200)
        if status.is_success() || status.is_redirection() {
            return Ok(());
        }

        Err(ScraperError::TaskDeletionFailed(format!(
            "Status: {}",
            status
        )))
    }
}
