# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2024-12-26

### Added

- #[sqly(select)] for custom SQL select expressions
- #[sqly(default, from)] for decoding table fields

### Fixed

- Rust 1.78.0 borrow checker error[E0716]
- Separate documentation pages

## [0.3.0] - 2024-10-17

### Added

- Support for nested objects through SQL JOIN clauses

## [0.2.0] - 2024-09-05

### Added

- Store information across separate derive invocations
- Generate basic select queries

## [0.1.0] - 2024-08-28

### Added

- Generate basic delete queries
- Generate basic insert queries
- Generate basic update queries
