# Contributing to TT-GLX-PARTITION

Thank you for your interest in contributing to TT-GLX-PARTITION! We welcome contributions from the community.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Pull Requests](#pull-requests)
- [Development Setup](#development-setup)
- [Style Guidelines](#style-guidelines)
  - [Git Commit Messages](#git-commit-messages)
  - [Rust Style Guide](#rust-style-guide)
- [Testing](#testing)
- [Additional Notes](#additional-notes)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When you create a bug report, please include as many details as possible using our bug report template.

**To submit a bug report:**

1. Use the GitHub issue tracker
2. Use the bug report template
3. Include detailed steps to reproduce
4. Describe the expected vs. actual behavior
5. Include your environment details (OS, Rust version, hardware)
6. Add relevant logs or error messages

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion:

1. Use the feature request template
2. Provide a clear and descriptive title
3. Explain the current behavior and the expected enhancement
4. Describe why this enhancement would be useful
5. Include examples or mockups if applicable

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following our style guidelines
3. **Test your changes** thoroughly
4. **Update documentation** if you've changed functionality
5. **Commit your changes** with clear, descriptive commit messages
6. **Push to your fork** and submit a pull request

**Pull Request Guidelines:**

- Fill out the pull request template completely
- Link any related issues
- Include screenshots or examples for UI changes
- Ensure all tests pass
- Keep pull requests focused on a single concern
- Request review from maintainers

## Development Setup

### Prerequisites

- Rust (stable channel recommended)
- Physical access to Tenstorrent Galaxy hardware (required for testing)
- Git

### Setup Instructions

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/tenstorrent/tt-glx-partition.git
   cd tt-glx-partition
   ```

3. **Build the project**:
   ```bash
   cargo build
   ```

4. **Run tests** (if available):
   ```bash
   cargo test
   ```

5. **Run the tool**:
   ```bash
   cargo run -- --help
   ```

### Important Development Notes

> **Note:** This tool requires direct access to physical PCIe devices and cannot be properly tested in VMs or containers. Ensure you have access to physical Tenstorrent Galaxy hardware for development and testing.

## Style Guidelines

### Git Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests liberally after the first line
- Consider starting commit messages with an applicable emoji for quick identification:
  - 🎨 `:art:` - Improving structure/format of the code
  - ⚡ `:zap:` - Improving performance
  - 🐛 `:bug:` - Fixing a bug
  - 📝 `:memo:` - Writing docs
  - ✨ `:sparkles:` - Introducing new features
  - ♻️ `:recycle:` - Refactoring code

### Rust Style Guide

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Run `cargo fmt` before committing to ensure consistent formatting
- Run `cargo clippy` and address any warnings
- Write meaningful variable and function names
- Comment complex logic, but prefer self-documenting code
- Keep functions focused and reasonably sized
- Handle errors appropriately; avoid unwrapping when possible

### Documentation

- Document all public APIs with doc comments (`///`)
- Include examples in doc comments where helpful
- Update the README.md if you change user-facing functionality
- Keep documentation clear, concise, and accurate

## Testing

- Write unit tests for new functionality
- Ensure existing tests pass before submitting a pull request
- Integration tests should be run on actual hardware when possible
- Include edge cases in your tests

Run tests with:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

## Additional Notes

### Hardware Requirements

This project requires specific Tenstorrent Galaxy hardware. If you don't have access to the hardware but want to contribute:

- Focus on documentation improvements
- Contribute to code structure and organization
- Help with error handling and logging
- Improve CLI interface and user experience

### Issue and Pull Request Labels

We use labels to organize and prioritize work:

- `bug` - Something isn't working
- `enhancement` - New feature or request
- `documentation` - Improvements or additions to documentation
- `good first issue` - Good for newcomers
- `help wanted` - Extra attention is needed
- `question` - Further information is requested

### Getting Help

If you need help or have questions:

1. Check existing issues and pull requests
2. Review the documentation
3. Open a new issue with the "question" label
4. Reach out to the maintainers

## Recognition

Contributors will be recognized in the project. We appreciate all contributions, whether they're code, documentation, bug reports, or feature suggestions!

Thank you for contributing to TT-GLX-PARTITION! 🎉
