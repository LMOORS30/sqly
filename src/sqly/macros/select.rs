#[allow(unused_imports)]
use super::*;



/// Applied to types which are defined to select rows from a table.
/// 
/// Implements [`Select`](Select).
/// 
/// <br>
/// 
/// ### Attribute Definition&ensp;<sub>(see [Attribute Notation](derive@Table#attribute-notation) and [Attribute Documentation](derive@Table#attribute-documentation))</sub>
/// 
/// ##### Struct Attributes:
/// `#[sqly((`[`table`](derive@Table#table)`)! (= `[`Path`](derive@Table#table)`)!)] // required`<br>
/// `#[sqly((`[`rename`](derive@Table#rename)`)? (= `[`String`](derive@Table#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`unchecked`](derive@Table#dev-attributes)`)?)]`<br>
/// `#[sqly((`[`print`](derive@Table#dev-attributes)`)?)]`<br>
/// `#[sqly((`[`debug`](derive@Table#dev-attributes)`)?)]`<br>
/// 
/// ##### Field Attributes:
/// `#[sqly((`[`column`](derive@Table#column)`)? (= `[`String`](derive@Table#column)`)!)]`<br>
/// `#[sqly((`[`rename`](derive@Table#rename)`)? (= `[`String`](derive@Table#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`value`](derive@Table#value)`)? (= `[`Expr`](derive@Table#value)`)!)]`<br>
/// `#[sqly((`[`infer`](derive@Table#infer)`)?)]`<br>
/// 
/// `#[sqly((`[`skip`](derive@Table#skip)`)?)]`
/// 
/// <br>
/// 
/// # Example
/// ```
/// use sqly::*; // traits
/// # type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// 
/// #[derive(Table)]
/// #[sqly(table = "books")]
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
