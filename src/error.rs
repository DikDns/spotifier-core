use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("Request to SPOT failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse HTML: {0}")]
    ParsingError(String),

    #[error("The SPOT session appears to have expired")]
    SessionExpired,

    #[error("Authentication failed. Please check your credentials.")]
    AuthenticationFailed,

    #[error("Could not find the login CSRF token on the page")]
    TokenNotFound,

    #[error("Could not find required element on the page: {0}")]
    ElementNotFound(String),
}

pub type Result<T> = std::result::Result<T, ScraperError>;
