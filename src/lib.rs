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
//! <br>
//! 
//! See [`#[derive(Table)]`](derive@Table) to browse the documentation or follow the chapters in the [examples](https://github.com/LMOORS30/sqly/tree/master/examples).
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
