use crate::cache::CacheBackend;
use crate::error::{Result, ScraperError};
use crate::models::{
    Course, DelayConfig, DetailCourse, Period, Semester, TopicDetail, TopicInfo, User,
};
use crate::parsers;
use rand::Rng;
use reqwest::cookie::{CookieStore, Jar};
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

/// The core client for interacting with the SPOT API.
///
/// This client handles authentication, period management, course/topic retrieval,
/// and task submissions with built-in protections like randomized delays and
/// User-Agent rotation to simulate human behavior.
pub struct SpotifierCoreClient {
    client: reqwest::Client,
    base_url: String,
    delay_config: DelayConfig,
    cookie_jar: Arc<Jar>,
    cache: Option<Arc<dyn CacheBackend>>,
    cache_prefix: Option<String>,
}

impl SpotifierCoreClient {
    /// Creates a new `SpotifierCoreClient` with default configuration.
    ///
    /// The default configuration includes randomized delays (1-3s) between requests.
    pub fn new() -> Self {
        Self::with_config(DelayConfig::default())
    }

    /// Creates a new `SpotifierCoreClient` with a custom `DelayConfig`.
    pub fn with_config(delay_config: DelayConfig) -> Self {
        let cookie_jar = Arc::new(Jar::default());

        let mut headers = HeaderMap::new();
        // Pick a random User-Agent at initialization
        let ua = MODERN_USER_AGENTS[rand::rng().random_range(0..MODERN_USER_AGENTS.len())];
        headers.insert(USER_AGENT, ua.parse().unwrap());

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .cookie_provider(Arc::clone(&cookie_jar))
            .default_headers(headers) // Use the headers
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://spot.upi.edu".to_string(),
            delay_config,
            cookie_jar,
            cache: None,
            cache_prefix: None,
        }
    }

    /// Sets the cache backend for the client.
    ///
    /// This allows storing session cookies and API responses to improve performance
    /// and avoid frequent SSO logins.
    pub fn set_cache(&mut self, cache: Arc<dyn CacheBackend>) {
        self.cache = Some(cache);
    }

    /// Sets a prefix/namespace for all cache keys.
    ///
    /// This is particularly useful for multi-user applications (like an API)
    /// to isolate cache data for different users (e.g., using a student identity).
    pub fn set_cache_prefix(&mut self, prefix: &str) {
        self.cache_prefix = Some(prefix.to_string());
    }

    fn get_cache_key(&self, key: &str) -> String {
        match &self.cache_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }

    /// Saves the current session cookies to a JSON file.
    ///
    /// This allows for session persistence across different runs of the application.
    pub async fn save_cookies(&self, path: &std::path::Path) -> Result<()> {
        let spot_url = "https://spot.upi.edu".parse().unwrap();
        let sso_url = "https://sso.upi.edu".parse().unwrap();

        let mut cookie_map = HashMap::new();
        if let Some(c) = self.cookie_jar.cookies(&spot_url) {
            cookie_map.insert("spot", c.to_str().unwrap_or_default().to_string());
        }
        if let Some(c) = self.cookie_jar.cookies(&sso_url) {
            cookie_map.insert("sso", c.to_str().unwrap_or_default().to_string());
        }

        let json = serde_json::to_string_pretty(&cookie_map).map_err(|e| {
            ScraperError::ParsingError(format!("Failed to serialize cookies: {}", e))
        })?;

        tokio::fs::write(path, json).await.map_err(|e| {
            ScraperError::ParsingError(format!("Failed to write cookie file: {}", e))
        })?;

        Ok(())
    }

    /// Loads session cookies from a JSON file.
    ///
    /// This populates the internal cookie jar, allowing the client to resume a session
    /// without re-authenticating through SSO.
    pub async fn load_cookies(&self, path: &std::path::Path) -> Result<()> {
        let json = tokio::fs::read_to_string(path).await.map_err(|e| {
            ScraperError::ParsingError(format!("Failed to read cookie file: {}", e))
        })?;

        let cookie_map: HashMap<String, String> = serde_json::from_str(&json).map_err(|e| {
            ScraperError::ParsingError(format!("Failed to deserialize cookies: {}", e))
        })?;

        let spot_url = "https://spot.upi.edu".parse().unwrap();
        let sso_url = "https://sso.upi.edu".parse().unwrap();

        if let Some(c) = cookie_map.get("spot") {
            for cookie in c.split(';') {
                self.cookie_jar.add_cookie_str(cookie.trim(), &spot_url);
            }
        }
        if let Some(c) = cookie_map.get("sso") {
            for cookie in c.split(';') {
                self.cookie_jar.add_cookie_str(cookie.trim(), &sso_url);
            }
        }

        Ok(())
    }

    /// Sets a new delay configuration for the client.
    pub fn set_delay_config(&mut self, config: DelayConfig) {
        self.delay_config = config;
    }

    fn get_random_ua(&self) -> &'static str {
        MODERN_USER_AGENTS[rand::rng().random_range(0..MODERN_USER_AGENTS.len())]
    }

    /// Waits for a random duration based on the current `DelayConfig`.
    async fn wait_random(&self) {
        if !self.delay_config.enabled {
            return;
        }

        let ms = rand::rng()
            .random_range(self.delay_config.min_delay_ms..=self.delay_config.max_delay_ms);
        sleep(std::time::Duration::from_millis(ms)).await;
    }

    /// Helper to perform a GET request with randomized delay and rotated User-Agent.
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

    /// Helper to perform a POST request with randomized delay and rotated User-Agent.
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

    /// Logs into SPOT using a student ID (NIM) and password through the SSO system.
    ///
    /// This involves a three-step process:
    /// 1. Fetching the SSO login page to retrieve the execution token.
    /// 2. Posting credentials to the SSO service.
    /// 3. Validating the final redirection back to the SPOT platform.
    pub async fn login(&self, nim: &str, password: &str) -> Result<()> {
        // Step 1: GET the login page to retrieve the "execution" token
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

        // Step 2: POST credentials to the SSO service
        let mut params = HashMap::new();
        params.insert("username", nim);
        params.insert("password", password);
        params.insert("execution", execution_token);
        params.insert("_eventId", "submit");

        let response = self
            .post_request(login_action_url.as_str(), &params)
            .await?;

        // Simulate human processing time after successful login
        if self.delay_config.enabled {
            let jitter_ms = rand::rng().random_range(2000..=5000);
            sleep(std::time::Duration::from_millis(jitter_ms)).await;
        }

        // Step 3: Verify the final redirection to SPOT
        let final_url = response.url().clone();
        if final_url.host_str() != Some("spot.upi.edu") {
            let error_body = response.text().await.unwrap_or_default();
            std::fs::write("login_fail.html", error_body).ok();

            return Err(ScraperError::AuthenticationFailed);
        }

        Ok(())
    }

    /// Internal helper to fetch HTML content from a specific path.
    async fn get_html(&self, path: &str) -> Result<String> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.get_request(&url).await?;

        if !response.url().path().starts_with(path) {
            return Err(ScraperError::SessionExpired);
        }

        Ok(response.text().await?)
    }

    /// Fetches the basic profile information of the currently logged-in user.
    pub async fn get_user_profile(&self) -> Result<User> {
        let html_content = self.get_html("/mhs").await?;
        parsers::user::parse_user_from_html(&html_content)
    }

    /// Fetches the list of all courses the user is enrolled in for the active period.
    ///
    /// This method is cached for 1 hour by default if a `CacheBackend` is configured.
    pub async fn get_courses(&self) -> Result<Vec<Course>> {
        let cache_key = self.get_cache_key("courses");
        if let Some(cache) = &self.cache {
            if let Some(cached_data) = cache.get(&cache_key).await {
                if let Ok(courses) = serde_json::from_str(&cached_data) {
                    return Ok(courses);
                }
            }
        }

        let html_content = self.get_html("/mhs").await?;
        let courses = parsers::courses::parse_courses_from_html(&html_content)?;

        if let Some(cache) = &self.cache {
            if let Ok(json) = serde_json::to_string(&courses) {
                let _ = cache.set(&cache_key, &json, 3600).await; // 1-hour expiration
            }
        }

        Ok(courses)
    }

    /// Fetches full details for a specific course, including its topics.
    pub async fn get_course_detail(&self, course: &Course) -> Result<DetailCourse> {
        // The href for the detail page is already stored in the Course struct
        let html_content: String = self.get_html(&course.href).await?;
        parsers::course_detail::parse_course_detail_from_html(&html_content, course.clone())
    }

    /// Fetches detailed information for a specific topic, including associated tasks.
    pub async fn get_topic_detail(&self, topic_info: &TopicInfo) -> Result<TopicDetail> {
        let href = topic_info.href.as_ref().ok_or_else(|| {
            ScraperError::ParsingError("TopicInfo does not have a valid href".to_string())
        })?;

        let course_id = topic_info.course_id.ok_or_else(|| {
            ScraperError::ParsingError("TopicInfo does not have a valid course_id".to_string())
        })?;
        let topic_id = topic_info.id.ok_or_else(|| {
            ScraperError::ParsingError("TopicInfo does not have a valid topic_id".to_string())
        })?;

        let html_content = self.get_html(href).await?;
        parsers::topic_detail::parse_topic_detail_from_html(&html_content, topic_id, course_id)
    }

    /// Convenience method to get course details using its unique ID.
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

    /// Convenience method to get topic details using course and topic IDs.
    pub async fn get_topic_detail_by_id(
        &self,
        course_id: u64,
        topic_id: u64,
    ) -> Result<TopicDetail> {
        let path = format!("/mhs/topik/{}/{}", course_id, topic_id);
        let html_content = self.get_html(&path).await?;
        parsers::topic_detail::parse_topic_detail_from_html(&html_content, topic_id, course_id)
    }

    /// Changes the active academic period/semester for the current session.
    pub async fn change_period(&self, year: u16, semester: Semester) -> Result<()> {
        let period = Period::new(year, semester);
        let path = format!("/adm/semester/{}", period.format());

        let response = self
            .get_request(&format!("{}{}", self.base_url, path))
            .await?;

        let status = response.status();

        // Success usually results in a 200 OK or a 302 redirect to /adm
        if status.is_success() || status.is_redirection() {
            let final_url = response.url();
            if final_url.path() == "/adm" || final_url.path().starts_with("/adm") {
                return Ok(());
            }
        }

        // A 500 error typically indicates an invalid or unsupported period format
        if status.as_u16() == 500 {
            return Err(ScraperError::InvalidPeriod(format!(
                "Period {} is not valid or unavailable in the system",
                period.format()
            )));
        }

        Err(ScraperError::ParsingError(format!(
            "Unexpected response while changing period. Status: {}",
            status
        )))
    }

    /// Retrieves the raw string representation of the current academic period.
    ///
    /// Example: "2025/2026 - Ganjil"
    pub async fn get_current_period_info(&self) -> Result<String> {
        let courses = self.get_courses().await?;

        if let Some(first_course) = courses.first() {
            Ok(first_course.academic_year.clone())
        } else {
            Err(ScraperError::ParsingError(
                "No courses found to determine the current academic period".to_string(),
            ))
        }
    }

    /// Internal helper to perform a POST request with a multipart form (used for file uploads).
    async fn multipart_request(
        &self,
        url: &str,
        form: multipart::Form,
    ) -> Result<reqwest::Response> {
        self.wait_random().await;
        let ua = self.get_random_ua();
        self.client
            .post(url)
            .header(USER_AGENT, ua)
            .multipart(form)
            .send()
            .await
            .map_err(ScraperError::from)
    }

    /// Submits a task to the SPOT platform.
    ///
    /// # Arguments
    /// * `course_id` - The unique identifier for the course.
    /// * `topic_id` - The unique identifier for the topic.
    /// * `task_id` - The unique identifier for the specific task.
    /// * `token` - The CSRF token required for the submission (extracted from the topic detail).
    /// * `content` - The text content or description for the submission.
    /// * `file_name` - Optional name for an attached file.
    /// * `file_data` - Optional bytes for the attached file.
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
            .multipart_request(&format!("{}/mhs/tugas_store", self.base_url), form)
            .await?;

        let status = response.status();

        // Successful submission usually results in a redirect back to the topic page
        if status.is_success() || status.is_redirection() {
            return Ok(());
        }

        Err(ScraperError::TaskSubmissionFailed(format!(
            "Server returned error status: {}",
            status
        )))
    }

    /// Deletes an existing task submission from the SPOT platform.
    ///
    /// # Arguments
    /// * `course_id` - The unique identifier for the course.
    /// * `topic_id` - The unique identifier for the topic.
    /// * `answer_id` - The ID of the submission to delete (found in the `Task`'s `answer` field).
    pub async fn delete_task_submission(
        &self,
        course_id: u64,
        topic_id: u64,
        answer_id: u64,
    ) -> Result<()> {
        let path = format!("/mhs/tugas_del/{}/{}/{}", course_id, topic_id, answer_id);
        let response = self
            .get_request(&format!("{}{}", self.base_url, path))
            .await?;

        let status = response.status();

        // Successful deletion usually results in a redirect back to the topic page
        if status.is_success() || status.is_redirection() {
            return Ok(());
        }

        Err(ScraperError::TaskDeletionFailed(format!(
            "Server returned error status: {}",
            status
        )))
    }
}
