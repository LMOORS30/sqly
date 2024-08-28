sqly
[<img alt="github.com" src="https://img.shields.io/badge/github.com-LMOORS30/sqly-5e728a?labelColor=343942&style=for-the-badge&logo=github" height="20">](https://github.com/LMOORS30/sqly)
[<img alt="crates.io" src="https://img.shields.io/badge/crates.io-sqly-5e888a?labelColor=343942&style=for-the-badge&logo=rust" height="20">](https://crates.io/crates/sqly)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-sqly-5e8a76?labelColor=343942&style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/sqly)
====

sqly is a lightweight macro system on top of [sqlx](https://github.com/launchbadge/sqlx), inspired by [ormx](https://github.com/NyxCode/ormx).

It works by generating common SQL queries and associated structs at compile time, letting the generated queries be checked and executed by sqlx.

This crate differs from ormx mainly by the added functionality of generating SQL `SELECT` queries with support for nested objects through SQL `JOIN` clauses. Additionally, `sqly::query!` macros can be used to further expand generated queries while still providing compile-time verification.

This functionality is still under development (see [Roadmap](#roadmap)).
<br>
<br>
##### Cargo.toml
```toml
[dependencies.sqly]
version = "0.1.0"
features = ["postgres"]

[dependencies.sqlx]
version = "0.8.0"
default-features = false
features = ["macros", "postgres"]
```

##### Features
`unchecked`&ensp;—&ensp;disable compile-time checking<br>
` postgres`&ensp;—&ensp;generate queries for PostgreSQL<br>
`   sqlite`&ensp;—&ensp;generate queries for SQLite (not supported)<br>
`    mysql`&ensp;—&ensp;generate queries for MySQL (not supported)

Currently only postgres is supported.

<br>

### Usage
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-sqly-5e8a76?labelColor=343942&style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/sqly)

<br>

### Roadmap
- [x] Basic `DELETE` queries
- [x] Basic `INSERT` queries
- [x] Basic `UPDATE` queries
- [ ] Storing information across separate `#[derive]` invocations
- [ ] Basic `SELECT` queries
- [ ] `sqly::query!` macros to extend generated queries
- [ ] `#[sqly(alias)]` attribute and optional override in macros
- [ ] `#[sqly(default, from, try_from, flatten)]` attributes for select
- [ ] `#[sqly(optional)]` attribute for optional update and insert fields
- [ ] `#[sqly(returning)]` attribute for generating SQL `RETURNING` clauses
- [ ] `#[sqly(foreign)]` attributes for nested objects through SQL `JOIN` clauses
- [ ] Implementation of bulk operations for `&[T]`
- [ ] Support for SQL `DISTINCT` clause in select
- [ ] Support for including constants in the SQL
- [ ] Support for filtering on nullable columns
- [ ] Support for foreign macro attributes
- [ ] Support for generics
- [ ] ... ?

<br>

#### License

<sup>
Licensed under either of
<a href="LICENSE-APACHE">Apache License, Version 2.0</a> or
<a href="LICENSE-MIT">MIT license</a>
at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
