#[allow(unused_imports)]
use super::*;



/// Applied to types which are defined to delete rows from a table.
/// 
/// Implements [`DeleteImpl`](DeleteImpl). Might also implement [`DeleteCheck`](DeleteCheck) and [`Delete`](Delete).
/// 
/// <br>
/// 
/// ### Attribute Definition&ensp;<sub>(see [Attribute Notation](docs::attr::note) and [Attribute Documentation](docs::attr))</sub>
/// 
/// ##### Struct Attributes:
/// `#[sqly((`[`table`](docs::attr#table)`)! (= `[`Path`](docs::attr#table)` | `[`String`](docs::attr#table)`)!)] // required`<br>
/// `#[sqly((`[`rename_all`](docs::attr#rename)`)? (= `[`String`](docs::attr#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`dynamic`](docs::attr#dynamic)`)?)]`<br>
/// `#[sqly((`[`optional`](docs::attr#optional)`)?)]`<br>
/// `#[sqly((`[`filter`](docs::attr#filter)`)* (= `[`String`](docs::attr#filter)`)+)]`<br>
/// `#[sqly((`[`returning`](docs::attr#returning)`)? (= `[`Path`](docs::attr#returning)`? { `[`Ident`](docs::attr#returning)`,+ }? )?)]`<br>
/// 
/// `#[sqly((`[`crate`](docs::attr#codegen)`)? (= `[`Path`](docs::attr#codegen)`)!)]`<br>
/// `#[sqly((`[`unchecked`](docs::attr#codegen)`)? (= `[`query`](docs::attr#codegen)` | `[`types`](docs::attr#codegen)`)?)]`<br>
/// 
/// `#[sqly((`[`print`](docs::attr#development)`)? (= `[`panic`](docs::attr#development)` | `[`warn`](docs::attr#development)` | `[`stdout`](docs::attr#development)` | `[`stderr`](docs::attr#development)`)?)]`<br>
/// `#[sqly((`[`debug`](docs::attr#development)`)? (= `[`panic`](docs::attr#development)` | `[`warn`](docs::attr#development)` | `[`stdout`](docs::attr#development)` | `[`stderr`](docs::attr#development)`)?)]`<br>
/// 
/// ##### Field Attributes:
/// `#[sqly((`[`column`](docs::attr#column)`)? (= `[`String`](docs::attr#column)`)!)]`<br>
/// `#[sqly((`[`rename`](docs::attr#rename)`)? (= `[`String`](docs::attr#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`filter`](docs::attr#filter)`)* (= `[`String`](docs::attr#filter)`)+)]`<br>
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
/// #[derive(Delete)]
/// #[sqly(table = Book)]
/// struct DeleteBook {
///     id: i32, // all delete fields are keys
///     #[sqly(skip)]
///     info: &'static str, // except when skipped
/// }
/// 
/// async fn delete_book(book_id: i32, db: &sqlx::PgPool) -> Result<()> {
///     Book::delete(&DeleteBook {
///         id: book_id, // key
///         info: "..." // ignored
///     }) // returns `sqlx::query::Query`
///     .execute(db)
///     .await?;
///     Ok(())
/// }
/// ```
pub use sqly_macros::Delete;
