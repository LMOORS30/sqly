//! sqly is a lightweight macro system on top of [sqlx].
//! 
//! [![github-com]](https://github.com/LMOORS30/sqly)&ensp;[![crates-io]](https://crates.io/crates/sqly)&ensp;[![docs-rs]](crate)
//! 
//! [github-com]: https://img.shields.io/badge/github.com-LMOORS30/sqly-5e728a?labelColor=505050&style=for-the-badge&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-sqly-5e888a?labelColor=505050&style=for-the-badge&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-sqly-5e8a76?labelColor=505050&style=for-the-badge&logo=docs.rs
//! 
//! It works by generating common SQL queries and associated structs at compile time, letting the generated queries be checked and executed by sqlx. Additionally, [`sqly::query!`](query!) macros can be used to further expand generated queries while still providing compile-time verification.
//! 
//! This functionality is still under development (see [Roadmap](https://github.com/LMOORS30/sqly#roadmap)).
//! <br>
//! <br>
//! ##### Cargo.toml
//! ```toml
//! [dependencies.sqly]
//! version = "0.1.0"
//! features = ["postgres"]
//! 
//! [dependencies.sqlx]
//! version = "0.8.0"
//! default-features = false
//! features = ["postgres", "macros"]
//! 
//! [profile.dev.package.sqlx-macros]
//! opt-level = 3
//! 
//! [profile.dev.package.sqly-macros]
//! opt-level = 3
//! ```
//! 
//! ##### Features
//! `unchecked`&ensp;—&ensp;disable compile-time checking<br>
//! ` postgres`&ensp;—&ensp;generate queries for PostgreSQL<br>
//! `   sqlite`&ensp;—&ensp;generate queries for SQLite (not supported)<br>
//! `    mysql`&ensp;—&ensp;generate queries for MySQL (not supported)
//! 
//! Currently only postgres is supported.
//! 
//! <br>
//! 
//! # Example
//! ```
//! use sqly::*;
//! # type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
//! 
//! #[derive(Table)]
//! #[sqly(table = "books")]
//! struct Book {
//!     id: i32,
//!     title: String,
//! }
//! 
//! #[derive(Table)]
//! #[sqly(insert, update, delete = DeleteAllPages)]
//! #[sqly(table = "pages")]
//! struct Page {
//!     #[sqly(key)]
//!     book_id: i32,
//!     #[sqly(key = update)]
//!     page_number: i32,
//!     content: String,
//!     #[sqly(skip = update)]
//!     read: bool,
//! }
//! 
//! #[derive(Update)]
//! #[sqly(table = Page)]
//! struct MarkAsRead {
//!     #[sqly(key)]
//!     book_id: i32,
//!     #[sqly(key)]
//!     page_number: i32,
//!     read: bool,
//! }
//! 
//! async fn test(book: &Book, db: &sqlx::PgPool) -> Result<()> {
//!     Page::insert(&InsertPage {
//!         book_id: book.id,
//!         page_number: 1,
//!         content: "The Wrong Content".into(),
//!         read: false,
//!     })
//!     .execute(db)
//!     .await?;
//! 
//!     Page::update(&UpdatePage {
//!         book_id: book.id,
//!         page_number: 1,
//!         content: "The Right Content".into(),
//!     })
//!     .execute(db)
//!     .await?;
//! 
//!     Page::update(&MarkAsRead {
//!         book_id: book.id,
//!         page_number: 1,
//!         read: true,
//!     })
//!     .execute(db)
//!     .await?;
//! 
//!     Page::delete(&DeleteAllPages {
//!         book_id: book.id,
//!     })
//!     .execute(db)
//!     .await?;
//! 
//!     Ok(())
//! }
//! ```
//! Currently only simple `DELETE`, `INSERT` and`UPDATE` queries are supported, `SELECT` is documented but not yet implemented. Any usage of [`#[derive(Select)]`](derive@Select) or [`#[sql(select)]`](derive@Table#select) will yield a compile error.
//! 
//! See [`#[derive(Table)]`](derive@Table) to get started.
//! 
//! <br>



#[cfg(feature = "sqlite")]
compile_error!("sqlite is currently not supported");
#[cfg(feature = "mysql")]
compile_error!("mysql is currently not supported");

#[cfg(feature = "postgres")]
mod sqly;

#[cfg(feature = "postgres")]
pub use sqly::*;
