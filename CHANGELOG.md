# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-01-27

### Added
- Print as panic, warn, stdout or stderr
- Add type override escape hatch to `#[sqly(column)]`
- `#[sqly(unchecked = types)]` for sqlx `_unchecked!` variants
- `#[sqly(default)]` with rust expressions
- `#[sqly(try_from)]` for fallible decoding conversion
- Error reporting for invalid paths
- Parentheses for attribute values
- Example chapters
- Trybuild testing
- Cargo check build test

### Changed
- Serde feature for double_option
- Add manual assignment to `#[sqly(update)]`
- Remove surrounding quotes from `#[sqly(table)]`
- Rename to `#[sqly(rename_all)]`
- Add `"$table"` to `#[sqly(filter)]`
- Rework `#[sqly(dynamic)]` for consistency
- `#[sqly(keyless)]` for consistency
- `automatically_derived` constant

### Fixed
- Consistent error reporting
- Consistent field resolution for `#[sqly(returning)]`
- Correct output for queries with static filter
- Less verbose queries for simple cases
- Correct output for tables with unchecked types
- Prevent `non_snake_case` warnings

## [0.4.0] - 2025-04-10

### Added
- `#[sqly(insert, update)]` for custom SQL value expressions
- `#[sqly(filter)]` for custom SQL filter expressions
- Support `#[sqly(table)]` as string for applicable derives
- `sqlx` as re-export and `#[sqly(crate)]` for crate path
- `#[sqly(optional)]` for dynamic SQL through optional fields
- `#[sqly(returning)]` for SQL `RETURNING` clause

### Changed
- Remove manual alias from `#[sqly(select)]`
- `Sqly::Checked` trait and `#[sqly(value)]` with self
- `Sqly::Flat` trait for optional `#[sqly(flat)]` struct
- Rename and generalize `#[sqly(foreign_)]` attributes
- Replace `unchecked` feature with default `checked` feature
- Opt-in with `#[sqly(from_row, from_flat, flat_row)]`
- Rename to `#[sqly(skip = from_row)]`

### Fixed

- Consistent whitespacing for string attributes
- Disallow `#[sqly(foreign, select)]`
- Raw identifiers as columns
- Bind args as tuple
- Additional query traits

## [0.3.1] - 2024-12-26

### Added

- `#[sqly(select)]` for custom SQL select expressions
- `#[sqly(default, from)]` for decoding table fields

### Fixed

- Rust 1.78.0 borrow checker `error[E0716]`
- Separate documentation pages

## [0.3.0] - 2024-10-17

### Added

- Support for nested objects through SQL `JOIN` clauses

## [0.2.0] - 2024-09-05

### Added

- Store information across separate derive invocations
- Generate basic `select` queries

## [0.1.0] - 2024-08-28

### Added

- Generate basic `delete` queries
- Generate basic `insert` queries
- Generate basic `update` queries
