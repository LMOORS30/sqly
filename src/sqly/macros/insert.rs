#[allow(unused_imports)]
use super::*;



/// Applied to types which are defined to insert rows into a table.
/// 
/// Implements [`InsertImpl`](InsertImpl). Might also implement [`InsertCheck`](InsertCheck) and [`Insert`](Insert).
/// 
/// <br>
/// 
/// ### Attribute Definition&ensp;<sub>(see [Attribute Notation](docs::attr::note) and [Attribute Documentation](docs::attr))</sub>
/// 
/// ##### Struct Attributes:
/// `#[sqly((`[`table`](docs::attr#table)`)! (= `[`Path`](docs::attr#table)` | `[`String`](docs::attr#table)`)!)] // required`<br>
/// `#[sqly((`[`rename`](docs::attr#rename)`)? (= `[`String`](docs::attr#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`dynamic`](docs::attr#dynamic)`)?)]`<br>
/// `#[sqly((`[`optional`](docs::attr#optional)`)?)]`<br>
/// `#[sqly((`[`returning`](docs::attr#returning)`)? (= `[`Path`](docs::attr#returning)`? { `[`Ident`](docs::attr#returning)`,+ }? )?)]`<br>
/// 
/// `#[sqly((`[`unchecked`](docs::attr#codegen)`)?)]`<br>
/// `#[sqly((`[`crate`](docs::attr#codegen)`)? (= `[`Path`](docs::attr#codegen)`)!)]`<br>
/// `#[sqly((`[`print`](docs::attr#development)`)? (= `[`panic`](docs::attr#development)` | `[`warn`](docs::attr#development)` | `[`stdout`](docs::attr#development)` | `[`stderr`](docs::attr#development)`)?)]`<br>
/// `#[sqly((`[`debug`](docs::attr#development)`)? (= `[`panic`](docs::attr#development)` | `[`warn`](docs::attr#development)` | `[`stdout`](docs::attr#development)` | `[`stderr`](docs::attr#development)`)?)]`<br>
/// 
/// ##### Field Attributes:
/// `#[sqly((`[`column`](docs::attr#column)`)? (= `[`String`](docs::attr#column)`)!)]`<br>
/// `#[sqly((`[`rename`](docs::attr#rename)`)? (= `[`String`](docs::attr#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`insert`](docs::attr#insert-1)`)* (= `[`String`](docs::attr#insert-1)`)+)]`<br>
/// `#[sqly((`[`optional`](docs::attr#optional)`)? (= `[`bool`](docs::attr#optional)`)?)]`<br>
/// `#[sqly((`[`value`](docs::attr#value)`)? (= `[`Expr`](docs::attr#value)`)!)]`<br>
/// `#[sqly((`[`infer`](docs::attr#infer)`)?)]`<br>
/// 
/// `#[sqly((`[`skip`](docs::attr#skip)`)?)]`
/// 
/// <br>
/// 
/// # Example
/// ```
/// use sqly::derive::*; // traits
/// # type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// 
/// #[derive(Table)]
/// #[sqly(table = "books")]
/// struct Book {
///     // ...
///     // these fields are ignored
/// }
/// 
/// #[derive(Insert)]
/// #[sqly(table = Book)]
/// struct InsertBook {
///     title: String, // all insert fields are values
///     #[sqly(skip)]
///     info: &'static str, // except when skipped
/// }
/// 
/// async fn insert_book(title: String, db: &sqlx::PgPool) -> Result<()> {
///     Book::insert(&InsertBook {
///         title: title, // value
///         info: "..." // ignored
///     }) // returns `sqlx::query::Query`
///     .execute(db)
///     .await?;
///     Ok(())
/// }
/// ```
pub use sqly_macros::Insert;
