# Contributing to up-man

Thank you for considering contributing to up-man! This document provides guidelines and conventions for contributing to the project.

## Development Process

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Add tests for your changes
5. Run the existing tests to make sure nothing breaks
6. Submit a pull request

## Conventional Commits

We follow the [Conventional Commits](https://www.conventionalcommits.org/) standard to make the commit history more readable and to automate the versioning and release process.

### Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes only
- `style`: Changes that do not affect the meaning of the code (formatting)
- `refactor`: Code changes that neither fix a bug nor add a feature
- `perf`: Code changes that improve performance
- `test`: Adding or modifying tests
- `build`: Changes to the build process or tools
- `ci`: Changes to CI configuration
- `chore`: Other changes that don't modify src or test files

### Scope

The scope should be the area of the codebase affected by the change, such as:

- `cli`: Command-line interface
- `config`: Configuration handling
- `runner`: Package manager update execution
- `detection`: Package manager detection
- `output`: Logging and output formatting

### Examples

```
feat(runner): add parallel update support
fix(config): handle paths with spaces correctly
docs(readme): update installation instructions
test(detection): add tests for package manager detection
```

## Versioning

We use [Semantic Versioning](https://semver.org/) with the following convention:

- `MAJOR`: Incompatible API changes
- `MINOR`: Backwards-compatible functionality additions
- `PATCH`: Backwards-compatible bug fixes

## Code Style

- Use 4 spaces for indentation
- Keep lines to a reasonable length (around 100 characters)
- Use meaningful variable names
- Add documentation comments for public APIs
- Follow Rust's official style guidelines

## Testing

- Write tests for new functionality
- Make sure all tests pass before submitting a PR
- Add both unit tests and integration tests where appropriate
- Consider edge cases in your tests

## Documentation

- Update the README.md when adding new features
- Document public APIs with rustdoc comments
- Update the TODO.md when implementing planned features

Thank you for contributing to up-man!
