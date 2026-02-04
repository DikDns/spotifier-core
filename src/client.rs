use crate::error::{Result, ScraperError};
use crate::models::{Course, DetailCourse, TopicDetail, TopicInfo, User};
use crate::parsers;
use reqwest::cookie::Jar;
use reqwest::header::{HeaderMap, USER_AGENT};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::Arc;

const SSO_LOGIN_PAGE_URL: &str =
    "https://sso.upi.edu/cas/login?service=https://spot.upi.edu/beranda";

pub struct SpotifierCoreClient {
    client: reqwest::Client,
    base_url: String,
}

impl SpotifierCoreClient {
    pub fn new() -> Self {
        let cookie_jar = Arc::new(Jar::default());

        // --- CHANGE 1: Add a default User-Agent header ---
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.0.0 Safari/537.36"
                .parse()
                .unwrap(),
        );

        let client = reqwest::Client::builder()
            .cookie_provider(cookie_jar)
            .default_headers(headers) // Use the headers
            .build()
            .unwrap();

        Self {
            client,
            base_url: "https://spot.upi.edu".to_string(),
        }
    }

    /// Logs into SPOT using a student ID (NIM) and password.
    pub async fn login(&self, nim: &str, password: &str) -> Result<()> {
        // --- STEP 1: GET the login page to get the "execution" token ---
        let response = self.client.get(SSO_LOGIN_PAGE_URL).send().await?;

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
            .client
            .post(login_action_url)
            .form(&params)
            .send()
            .await?;

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
        let response = self.client.get(&url).send().await?;

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
        let path = format!("/mhs/materi/{}/{}", course_id, topic_id);
        let html_content = self.get_html(&path).await?;
        parsers::topic_detail::parse_topic_detail_from_html(&html_content, topic_id, course_id)
    }
}
