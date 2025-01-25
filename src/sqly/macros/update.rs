#[allow(unused_imports)]
use super::*;



/// Applied to types which are defined to update rows in a table.
/// 
/// Implements [`Update`](Update).
/// 
/// <br>
/// 
/// ### Attribute Definition&ensp;<sub>(see [Attribute Notation](docs::attr::note) and [Attribute Documentation](docs::attr))</sub>
/// 
/// ##### Struct Attributes:
/// `#[sqly((`[`table`](docs::attr#table)`)! (= `[`Path`](docs::attr#table)`)!)] // required`<br>
/// `#[sqly((`[`rename`](docs::attr#rename)`)? (= `[`String`](docs::attr#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`unchecked`](docs::attr#dev-attributes)`)?)]`<br>
/// `#[sqly((`[`print`](docs::attr#dev-attributes)`)?)]`<br>
/// `#[sqly((`[`debug`](docs::attr#dev-attributes)`)?)]`<br>
/// 
/// ##### Field Attributes:
/// `#[sqly((`[`column`](docs::attr#column)`)? (= `[`String`](docs::attr#column)`)!)]`<br>
/// `#[sqly((`[`rename`](docs::attr#rename)`)? (= `[`String`](docs::attr#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`update`](docs::attr#update-1)`)* (= `[`String`](docs::attr#update-1)`)+)]`<br>
/// `#[sqly((`[`value`](docs::attr#value)`)? (= `[`Expr`](docs::attr#value)`)!)]`<br>
/// `#[sqly((`[`infer`](docs::attr#infer)`)?)]`<br>
/// 
/// `#[sqly((`[`skip`](docs::attr#skip)`)?)]`<br>
/// `#[sqly((`[`key`](docs::attr#key)`)?)]`
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
/// #[derive(Update)]
/// #[sqly(table = Book)]
/// struct UpdateBook {
///     #[sqly(key)] // update keys must be specified
///     id: i32,
///     title: String, // other fields are values
///     #[sqly(skip)]
///     info: &'static str, // except when skipped
/// }
/// 
/// async fn update_book(book_id: i32, title: String, db: &sqlx::PgPool) -> Result<()> {
///     Book::update(&UpdateBook {
///         id: book_id, // key
///         title: title, // value
///         info: "..." // ignored
///     }) // returns `sqlx::query::Query`
///     .execute(db)
///     .await?;
///     Ok(())
/// }
/// ```
pub use sqly_macros::Update;
