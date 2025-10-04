use crate::error::{Result, ScraperError};
use crate::models::User;
use scraper::{Html, Selector};

/// Parses the HTML of the main student dashboard page to extract user info.
pub fn parse_user_from_html(html: &str) -> Result<User> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(".user-profile .profile-text").unwrap();

    let profile_element = document
        .select(&selector)
        .next()
        .ok_or_else(|| ScraperError::ElementNotFound("User profile text element".to_string()))?;

    let profile_text = profile_element.text().collect::<String>();
    let parts: Vec<&str> = profile_text.trim().split_whitespace().collect();

    let nim = match parts.last() {
        Some(n) => n.to_string(),
        None => return Err(ScraperError::ParsingError("Could not extract NIM.".to_string())),
    };

    let name = parts[..parts.len() - 1].join(" ");

    if name.is_empty() {
        return Err(ScraperError::ParsingError("Could not extract user name.".to_string()));
    }

    Ok(User { name, nim })
}
