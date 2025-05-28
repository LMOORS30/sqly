#[allow(unused_imports)]
use super::*;



/// Applied to types which are defined to select rows from a table.
/// 
/// Implements [`SelectImpl`](SelectImpl). Might also implement [`SelectCheck`](SelectCheck) and [`Select`](Select).
/// 
/// <br>
/// 
/// ### Attribute Definition&ensp;<sub>(see [Attribute Notation](docs::attr::note) and [Attribute Documentation](docs::attr))</sub>
/// 
/// ##### Struct Attributes:
/// `#[sqly((`[`table`](docs::attr#table)`)! (= `[`Path`](docs::attr#table)`)!)] // required`<br>
/// `#[sqly((`[`rename`](docs::attr#rename)`)? (= `[`String`](docs::attr#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`dynamic`](docs::attr#dynamic)`)?)]`<br>
/// `#[sqly((`[`optional`](docs::attr#optional)`)?)]`<br>
/// `#[sqly((`[`filter`](docs::attr#filter)`)* (= `[`String`](docs::attr#filter)`)+)]`<br>
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
/// #[sqly(from_row)] // required
/// struct Book {
///     // ...
///     // these fields will be selected
/// }
/// 
/// #[derive(Select)]
/// #[sqly(table = Book)]
/// struct SelectBook {
///     title: String, // all select fields are keys
///     #[sqly(skip)]
///     info: &'static str, // except when skipped
/// }
/// 
/// async fn select_books(title: String, db: &sqlx::PgPool) -> Result<Vec<Book>> {
///     let res = Book::select(&SelectBook {
///         title: title, // key
///         info: "..." // ignored
///     }) // returns `sqlx::query::Map`
///     .fetch_all(db)
///     .await?;
///     Ok(res)
/// }
/// ```
pub use sqly_macros::Select;
