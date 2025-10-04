use std::collections::HashMap;
use scraper::{Html, Selector};
use crate::error::{Result, ScraperError};
use crate::models::User;
use crate::parsers;
use reqwest::cookie::Jar;
use std::sync::Arc;
use reqwest::header::{HeaderMap, USER_AGENT};

const SSO_LOGIN_PAGE_URL: &str = "https://sso.upi.edu/cas/login?service=https://spot.upi.edu/beranda";

pub struct SpotClient {
    client: reqwest::Client,
    base_url: String,
}

impl SpotClient {
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
        let response = self.client.post(login_action_url)
            .form(&params)
            .send()
            .await?;

        // --- STEP 3: Verify the final redirection URL ---
        let final_url = response.url().clone();
        if final_url.host_str() != Some("spot.upi.edu") {
            let error_body = response.text().await.unwrap_or_default();
            std::fs::write("login_fail.html", error_body).ok();
            println!("Login failed. Check login_fail.html for details. The final URL was: {}", final_url);

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
}
