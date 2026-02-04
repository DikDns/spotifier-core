// Declare all our modules
mod client;
mod error;
mod models;
mod parsers;

// Publicly export the parts of our library that users will need
pub use client::SpotifierCoreClient;
pub use error::{Result, ScraperError};
pub use models::*; // Exposes all structs like User, Course, etc.
