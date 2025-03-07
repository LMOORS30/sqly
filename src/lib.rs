//! sqly is a lightweight macro system on top of [sqlx].
//! 
//! See the [README](https://github.com/LMOORS30/sqly#sqly) for additional information, [Installation](https://github.com/LMOORS30/sqly#cargotoml) and [Features](https://github.com/LMOORS30/sqly#features).
//! 
//! [![github-com]](https://github.com/LMOORS30/sqly)&ensp;[![crates-io]](https://crates.io/crates/sqly)&ensp;[![docs-rs]](crate)
//! 
//! [github-com]: https://img.shields.io/badge/github.com-LMOORS30/sqly-5e728a?labelColor=505050&style=for-the-badge&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-sqly-5e888a?labelColor=505050&style=for-the-badge&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-sqly-5e8a76?labelColor=505050&style=for-the-badge&logo=docs.rs
//! 
//! # Example
//! ```
//! use sqly::derive::*;
//! # type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
//! 
//! #[derive(Table)]
//! #[sqly(table = "books")]
//! struct Book {
//!     #[sqly(key)]
//!     id: i32,
//!     title: String,
//! }
//! 
//! #[derive(Table)]
//! #[sqly(table = "pages")]
//! #[sqly(insert, update, select)]
//! #[sqly(delete = DeleteAllPages)]
//! struct Page {
//!     #[sqly(key, foreign)]
//!     book: Book,
//!     #[sqly(key, skip = delete)]
//!     page_number: i32,
//!     content: String,
//!     #[sqly(skip = update)]
//!     read: bool,
//! }
//! 
//! #[derive(Select)]
//! #[sqly(table = Page)]
//! struct GetBookPages {
//!     book_id: i32,
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
//!     // struct instantiation will likely be done externally (for example by serde)
//!     // and the structs should be passed as parameters (e.g. `page: &InsertPage`)
//!     // this syntax is less ideal and only used for the sake of this example
//! 
//!     Page::insert(&InsertPage { // insert a new page
//!         book_id: book.id,
//!         page_number: 1,
//!         content: "The Wrong Content".into(),
//!         read: false,
//!     })
//!     .execute(db)
//!     .await?;
//! 
//!     Page::update(&UpdatePage { // update the page content
//!         book_id: book.id,
//!         page_number: 1,
//!         content: "The Right Content".into(),
//!     })
//!     .execute(db)
//!     .await?;
//! 
//!     Page::update(&MarkAsRead { // mark the page as read
//!         book_id: book.id,
//!         page_number: 1,
//!         read: true,
//!     })
//!     .execute(db)
//!     .await?;
//! 
//!     let page = Page::select(&SelectPage { // select the updated page
//!         book_id: book.id,
//!         page_number: 1,
//!     })
//!     .fetch_one(db)
//!     .await?;
//!     assert_eq!(page.read, true); // confirm it is marked as read
//!     assert_eq!(page.book.title, book.title); // the book is also fetched
//! 
//!     Page::delete(&DeleteAllPages { // delete all pages from the book
//!         book_id: page.book.id,
//!     })
//!     .execute(db)
//!     .await?;
//!
//!     let pages = Page::select(&GetBookPages { // get all pages from the book
//!         book_id: page.book.id,
//!     })
//!     .fetch_all(db)
//!     .await?;
//!     assert!(pages.is_empty()); // confirm no pages are left
//! 
//!     Ok(())
//! }
//! ```
//! <br>
//! 
//! See [`#[derive(Table)]`](derive@Table) to get started.
//! 
//! <br>



#[cfg(feature = "mariadb")]
compile_error!("MariaDB is currently not supported");
#[cfg(feature = "sqlite")]
compile_error!("SQLite is currently not supported");
#[cfg(feature = "mysql")]
compile_error!("MySQL is currently not supported");

#[cfg(feature = "postgres")]
mod sqly;

#[cfg(feature = "postgres")]
pub use sqly::*;
