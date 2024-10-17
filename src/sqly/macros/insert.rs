#[allow(unused_imports)]
use super::*;



/// Applied to types which are defined to insert rows into a table.
/// 
/// Implements [`Insert`](Insert).
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
