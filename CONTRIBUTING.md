# Contributing Guidelines 🤝

Thank you for your interest in contributing to **Spotifier Core**! Before you proceed, please read the following guidelines:

- [Contributing Guidelines 🤝](#contributing-guidelines-)
  - [Contributions](#contributions)
  - [Getting Started](#getting-started)
  - [Commit Guidelines](#commit-guidelines)
    - [Message Guidelines](#message-guidelines)
  - [Pull Request Policy](#pull-request-policy)
    - [When Merging](#when-merging)
  - [Developer Certificate of Origin 1.1](#developer-certificate-of-origin-11)

## Contributions

Everyone is welcome to contribute to Spotifier Core. This repository currently recognizes two types of contribution roles:

- **Contributors**: Individuals who create issues/PRs, comment on them, or contribute in any other way.
- **Collaborators**: Individuals who review issues/PRs, manage them, or actively participate in project discussions and decision-making.

## Getting Started

Follow these steps to prepare your local environment and submit your contributions:

1. Click the **Fork** button at the top right to copy the [Spotifier Core Repository](https://github.com/DikDns/spotifier-core/fork).

2. Clone your fork using SSH, GitHub CLI, or HTTPS.
   ```bash
   git clone git@github.com:<YOUR_USERNAME>/spotifier-core.git # SSH
   git clone https://github.com/<YOUR_USERNAME>/spotifier-core.git # HTTPS
   gh repo clone <YOUR_USERNAME>/spotifier-core # GitHub CLI
   ```

3. Navigate to the project directory.
   ```bash
   cd spotifier-core
   ```

4. **Prerequisites for .env**: Create a `.env` file in the project root to run integration tests.
   ```env
   SPOT_NIM=your_student_id
   SPOT_PASSWORD=your_password
   ```

5. Set up the upstream remote to keep your fork up-to-date.
   ```bash
   git remote add upstream git@github.com:DikDns/spotifier-core.git # SSH
   git remote add upstream https://github.com/DikDns/spotifier-core.git # HTTPS
   ```

6. Create a new branch for your work.
   ```bash
   git checkout -b feat/my-awesome-feature
   ```

7. Install dependencies and run initial tests.
   ```bash
   cargo build
   cargo test -- --nocapture
   ```

8. Make your changes. If you are unfamiliar with the code structure, please refer to the [API Guide](Wiki-Client-Guide).

9. Sync your branch with the upstream main branch.
   ```bash
   git fetch upstream
   git merge upstream/main
   ```

10. Run linting and formatting checks.
    ```bash
    cargo fmt --all -- --check
    cargo clippy -- -D warnings
    ```

11. Commit and push your changes to your fork.
    ```bash
    git add .
    git commit -m "feat: add my awesome feature" # Follow commit guidelines below
    git push -u origin feat/my-awesome-feature
    ```

> [!IMPORTANT]
> Before committing and opening a Pull Request, please read the [Commit Guidelines](#commit-guidelines) and [Pull Request Policy](#pull-request-policy) sections below.

12. Create a Pull Request.

> [!NOTE]
> Please avoid unnecessary rebase/updates with the `main` branch unless conflicts occur.

## Commit Guidelines

This project follows the [Conventional Commits][] specification. 

Commits should be signed. Read more about [Commit Signing][] here.

### Message Guidelines
- Commit messages must include a "type" (e.g., `feat`, `fix`, `docs`).
- Commit messages **should not** end with a period `.`.

## Pull Request Policy

This policy governs how contributions should be merged.

### When Merging
- All required status checks (CI) must pass.
- Ensure all discussions have been resolved.
- Pull requests consisting of multiple commits should be **squashed**.

## Developer Certificate of Origin 1.1

```
By contributing to this project, I certify that:

- (a) The contribution was created in whole or in part by me and I have the right to submit it under the open source license indicated in the file; or
- (b) The contribution is based upon previous work that, to the best of my knowledge, is covered under an appropriate open source license and I have the right under that license to submit that work with modifications, whether created in whole or in part by me, under the same open source license (unless I am permitted to submit under a different license), as indicated in the file; or
- (c) The contribution was provided directly to me by some other person who certified (a), (b) or (c) and I have not modified it.
- (d) I understand and agree that this project and the contribution are public and that a record of the contribution (including all personal information I submit with it, including my signature) is maintained indefinitely and may be redistributed consistent with this project or the open source license(s) involved.
```

[Conventional Commits]: https://www.conventionalcommits.org/
[Commit Signing]: https://docs.github.com/en/authentication/managing-commit-signature-verification/signing-commits
