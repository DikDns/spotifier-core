# Spotifier Core 🚀

A high-performance, asynchronous Rust library for interacting with the University of Education in Konoha's Learning Management System.

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## ✨ Features

- **Authentication**: Seamless SSO (Single Sign-On) integration.
- **Session Persistence**: Save and load cookies to/from JSON to avoid repetitive logins.
- **Human-like Behavior**: Built-in randomized delays and User-Agent rotation to stay under the radar.
- **Academic Management**: Change semesters/periods and fetch enrollments.
- **Content Retrieval**: Parse courses, learning topics, and instructional materials.
- **Task Lifecycle**: Submit assignments with file uploads and manage existing submissions.
- **Flexible Caching**: Extensible caching trait with a default atomic file-based implementation.

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
spotifier-core = { git = "https://github.com/DikDns/spotifier-core" }
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use spotifier_core::{SpotifierCoreClient, DelayConfig, FileCache};
use std::sync::Arc;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize client
    let mut client = SpotifierCoreClient::new();
    
    // 2. Setup caching (Optional but recommended)
    let cache = Arc::new(FileCache::new(".cache"));
    client.set_cache(cache);
    client.set_cache_prefix("2306012"); // Use NIM/ID as namespace

    // 3. Login or load session
    if Path::new("session.json").exists() {
        client.load_cookies(Path::new("session.json")).await?;
    } else {
        client.login("NIM", "PASSWORD").await?;
        client.save_cookies(Path::new("session.json")).await?;
    }

    // 4. Fetch your courses
    let courses = client.get_courses().await?;
    for course in courses {
        println!("Enrolled in: {}", course.name);
    }

    Ok(())
}
```

## 🛡️ Stealth Features

To prevent detection, `spotifier-core` implements:
- **Randomized Jitter**: Wait times (default 1-3s) between every network request.
- **Login Delay**: A longer 2-5s "think time" after a successful SSO login.
- **UA Rotation**: Randomly selects from a list of modern browser User-Agents for every session.

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
*Disclaimer: This project is intended for educational and personal automation purposes. Please use it responsibly and in accordance with your institution's terms of service.*
