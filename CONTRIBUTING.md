# Contributing to Spotifier Core 🤝

Thank you for your interest in contributing! We appreciate any help to make this library better.

## 🛠️ Development Environment

1.  **Rust**: Ensure you have the latest stable version of Rust installed.
2.  **Environment Variables**: Create a `.env` file in the root for integration tests:
    ```env
    SPOT_NIM=your_nim
    SPOT_PASSWORD=your_password
    ```

## 🧪 Testing

We rely heavily on integration tests to ensure logic remains correct despite platform changes.

```bash
# Run all tests (requires .env)
cargo test -- --nocapture

# Run specific feature tests
cargo test --test task_test
cargo test --test cache_test
```

## 📜 Coding Guidelines

- **English Only**: All code comments, documentation, and error messages must be in English.
- **Documentation**: Use `///` doc comments for all public structures and methods.
- **Small Commits**: Prefer atomic, focused commits with clear messages.
- **Stealth First**: Never bypass the `wait_random` logic for public API methods unless it is strictly for performance testing.

## 🚀 Pull Request Process

1.  Fork the repository.
2.  Create a branch for your feature or fix (`git checkout -b feat/my-cool-feature`).
3.  Ensure all tests pass and documentation is updated.
4.  Submit a Pull Request with a clear description of what changed.

## 💬 Communication

If you have questions, feel free to open a GitHub Issue.

---
Happy coding! 🚀
