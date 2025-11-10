# Contributing to CLI Shell (Rust)

Thank you for your interest in contributing to our Rust-based CLI shell project! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Submitting Changes](#submitting-changes)
- [Review Process](#review-process)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow:

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive criticism
- Accept feedback gracefully
- Prioritize the community and project health

## Getting Started

### Prerequisites

- Rust 1.70 or later (install from https://rustup.rs/)
- Git
- A GitHub account

### Finding Issues to Work On

1. Check the [Issues](https://github.com/fatalem0/MHS-Software-Design-25/issues) page
2. Look for issues labeled:
   - `good first issue` - Great for newcomers
   - `help wanted` - Contributions especially welcome
   - `bug` - Bug fixes needed
   - `enhancement` - New features or improvements

3. Comment on an issue to express interest before starting work

## Development Setup

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/MHS-Software-Design-25.git
cd MHS-Software-Design-25
```

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 3. Build the Project

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### 4. Run the Shell

```bash
./target/debug/cli-shell
# or
./target/release/cli-shell
```

## Project Structure

```
MHS-Software-Design-25/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── bin/                 # Custom command implementations
│   │   ├── echo.rs
│   │   ├── cat.rs
│   │   ├── pwd.rs
│   │   └── wc.rs
│   └── modules/
│       ├── command.rs       # Command data structure
│       ├── environment.rs   # Environment variable management
│       ├── init.rs          # Initialization and configuration
│       ├── repl.rs          # Read-Eval-Print Loop
│       ├── runner.rs        # Command execution engine
│       └── input/           # Input processing modules
│           ├── mod.rs
│           ├── input_processor.rs  # Main facade
│           ├── tokenizer.rs        # Tokenization
│           ├── expander.rs         # Variable expansion
│           ├── quote_handler.rs    # Quote processing
│           └── ...
├── tests/                   # Integration tests
├── Cargo.toml              # Dependencies and metadata
└── README.md
```

### Key Modules

- **Environment** - Type-safe environment variable container
- **Init** - Single source of truth for shell configuration
- **InputProcessor** - Facade for command parsing pipeline
- **Runner** - Executes commands (custom or system)
- **Repl** - Interactive shell loop

## Coding Standards

### Rust Style

Follow the official Rust style guide:

```bash
# Format code before committing
cargo fmt

# Check for common mistakes
cargo clippy
```

### Documentation

All public items should have documentation comments:

```rust
/// Brief description of the function.
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `arg1` - Description of arg1
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// // Example usage
/// let result = my_function(arg);
/// ```
pub fn my_function(arg: Type) -> ReturnType {
    // implementation
}
```

### Error Handling

- Use `Result` types for operations that can fail
- Provide descriptive error messages
- Use custom error types when appropriate (see `src/modules/input/errors.rs`)

### Naming Conventions

- `snake_case` for functions, variables, and modules
- `PascalCase` for types and traits
- `SCREAMING_SNAKE_CASE` for constants
- Descriptive names preferred over abbreviations

### Code Organization

- Keep functions focused and small (ideally < 50 lines)
- Group related functionality into modules
- Use private functions to break down complex logic
- Avoid deep nesting (max 3-4 levels)

## Testing Guidelines

### Writing Tests

Every significant feature or bug fix should include tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_description() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Test Types

1. **Unit Tests** - Test individual functions (in the same file as the code)
2. **Integration Tests** - Test module interactions (in `tests/` directory)
3. **Documentation Tests** - Examples in doc comments should be valid code

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests for specific module
cargo test --test integration_tests
```

### Test Coverage

Aim for good test coverage:

```bash
# Install coverage tool
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html
cargo llvm-cov --open
```

We aim for at least 80% code coverage for new features.

## Submitting Changes

### Commit Messages

Write clear, descriptive commit messages:

```
type: Brief description (50 chars or less)

More detailed explanation if needed. Wrap at 72 characters.
Explain what changed and why, not how.

- Use bullet points for multiple changes
- Reference issue numbers
- Keep commits atomic (one logical change per commit)
```

### Pull Request Process

1. **Ensure all tests pass**:
   ```bash
   cargo test
   cargo fmt --check
   cargo clippy
   ```

2. **Update documentation**:
   - Update README.md if needed
   - Add/update doc comments
   - Update ARCHITECTURE.md for structural changes

3. **Create Pull Request**:
   - Use a clear, descriptive title
   - Fill out the PR template
   - Reference related issues
   - Describe what changed and why
   - Include screenshots for UI changes

4. **PR Template**:
   ```markdown
   ## Description
   Brief description of changes
   
   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update
   
   ## Testing
   - [ ] All tests pass
   - [ ] Added new tests
   - [ ] Manual testing performed
   
   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Documentation updated
   - [ ] No new warnings
   - [ ] Tested on multiple platforms (if applicable)
   
   Fixes #(issue number)
   ```

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.

---

Thank you for contributing to make this project better! 🚀
