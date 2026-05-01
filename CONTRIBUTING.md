# Contributing to JailGuard

Thank you for your interest in contributing to JailGuard! We appreciate all contributions - whether they're bug reports, documentation improvements, new features, or research insights.

## Code of Conduct

Before contributing, please review our [Code of Conduct](CODE_OF_CONDUCT.md). We are committed to providing a welcoming and respectful environment for all contributors.

## How to Contribute

### 1. Reporting Bugs

**Found a bug?** Please report it by creating a GitHub issue. Before submitting:

- **Check existing issues** - Maybe it's already reported
- **Include details:**
  - Clear title describing the issue
  - Step-by-step reproduction instructions
  - Expected vs actual behavior
  - Rust version: `rustc --version`
  - JailGuard version or commit hash
  - OS and architecture
  - Stack trace or error messages

**Example:**
```
Title: Model inference crashes with empty embedding vector

Steps to reproduce:
1. Load trained model
2. Call predict() with empty embedding (384 zeros)
3. Observe panic

Expected: Should return confidence score or error
Actual: thread 'main' panicked at 'assertion failed: embedding.len() == 384'

Environment:
- Rust: 1.75.0
- JailGuard: main branch (commit abc123)
- OS: Ubuntu 22.04
```

### 2. Suggesting Features

**Have an idea?** We'd love to hear it!

- Create a GitHub issue with title "Feature: [your idea]"
- Explain the use case and why it's valuable
- Include examples of how it would be used
- Link to any relevant research or papers

### 3. Submitting Code

#### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/yfedoseev/jailguard
cd jailguard

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
cargo --version
rustc --version
```

#### Development Workflow

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/bug-number
   ```

2. **Make your changes** and test frequently:
   ```bash
   # Run all tests
   cargo test --all

   # Run specific test suite
   cargo test --test integration_comprehensive

   # Test with all features
   cargo test --all-features
   ```

3. **Format your code:**
   ```bash
   cargo fmt
   ```

4. **Run linter:**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

5. **Build documentation:**
   ```bash
   cargo doc --no-deps --open
   ```

6. **Commit with clear messages:**
   ```bash
   git commit -m "Add feature: [brief description]"
   # or
   git commit -m "Fix: [issue number] - [description]"
   ```

7. **Push and create a Pull Request:**
   ```bash
   git push origin feature/your-feature-name
   ```

#### PR Checklist

Before submitting a pull request, ensure:

- [ ] All tests pass: `cargo test --all`
- [ ] Code is formatted: `cargo fmt --check`
- [ ] No clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] New tests added for new functionality
- [ ] PR title is descriptive
- [ ] Description explains what and why
- [ ] Linked to any related issues

### 4. Documentation Improvements

Documentation is crucial! You can help by:

- **Fixing typos** in README, guides, or comments
- **Improving clarity** in existing documentation
- **Adding examples** for complex features
- **Translating** documentation to other languages
- **Documenting** undocumented features

**To update docs:**

```bash
# Edit the markdown file
vim docs/GETTING_STARTED.md

# Or inline code documentation
vim src/embedded.rs

# Build and check
cargo doc --no-deps --open
```

### 5. Research Contributions

Interested in research? Relevant areas:

- **Adversarial robustness** — test and improve attack resilience
- **New architectures** — propose and implement new detection approaches
- **Performance** — optimize for speed and memory
- **Evaluation** — benchmark against PINT, AgentDojo, DataSentinel, etc.

## Development Guidelines

### Code Style

- **Formatting:** Use `cargo fmt` (enforced in CI)
- **Linting:** Follow clippy rules (enforced in CI)
- **Comments:** Clear, concise, explain the "why"
- **Naming:**
  - `snake_case` for functions and variables
  - `PascalCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

### Testing Requirements

- **Unit tests:** For individual functions and logic
- **Integration tests:** For component interactions
- **Robustness tests:** For edge cases and adversarial inputs
- **Benchmarks:** For performance-critical code

**Test command:**
```bash
cargo test --all --verbose
```

### Documentation Requirements

- **Public APIs** must have doc comments:
  ```rust
  /// Predicts whether input is a prompt injection.
  ///
  /// # Arguments
  /// * `embedding` - 384-dimensional embedding vector
  ///
  /// # Returns
  /// Prediction confidence between 0.0 (benign) and 1.0 (injection)
  ///
  /// # Example
  /// ```
  /// let network = NeuralBinaryNetwork::new(0.01);
  /// let confidence = network.predict(&embedding);
  /// ```
  pub fn predict(&self, embedding: &[f32]) -> f32 { ... }
  ```

- **Complex logic** should have explanatory comments
- **Examples** should be runnable and correct

### Performance Considerations

- Target: <30ms inference on CPU, <5ms on GPU
- Use benchmarks: `cargo bench`
- Profile memory usage
- Optimize hot paths only (measure first!)

## Release Process

1. **Version bumping:** Follow [semver](https://semver.org/)
   - PATCH: bug fixes (1.1.1)
   - MINOR: new features, backward compatible (1.2.0)
   - MAJOR: breaking changes (2.0.0)

2. **Update CHANGELOG.md** with changes

3. **Update version in Cargo.toml**

4. **Create release PR** with changes

5. **Tag release:** `git tag -a v1.2.0 -m "Release v1.2.0"`

6. **Publish:** `cargo publish`

## Security

**Found a security vulnerability?** Please don't create a public issue. Instead, see [SECURITY.md](SECURITY.md) for responsible disclosure instructions.

## Questions?

- **General questions:** Create a GitHub discussion or issue
- **Development questions:** Comment on relevant issues
- **Security concerns:** Email security contact (see SECURITY.md)

---

## Recognition

Contributors are recognized in:
- GitHub contributors page
- Release notes (for significant contributions)

## Additional Resources

- **Getting started:** [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md)
- **API reference:** [docs/API.md](docs/API.md)
- **Architecture:** [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **Integration guide:** [docs/INTEGRATION_GUIDE.md](docs/INTEGRATION_GUIDE.md)
- **Examples:** [examples/README.md](examples/README.md)

---

**Thank you for contributing to JailGuard!** 🙏
