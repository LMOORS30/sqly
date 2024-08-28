#[allow(unused_imports)]
use super::*;



/// Applied to types which are defined to select rows from a table.
/// 
/// <br>
/// 
/// # Examples
/// ```compile_fail
/// use sqly::*;
/// # type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// 
/// #[derive(Table)]
/// #[sqly(table = "books")]
/// struct Book {
///     // ...
/// }
/// 
/// #[derive(Select)]
/// #[sqly(table = Book)]
/// #[sqly(table_name = "books")] // currently required, soon obsolete
/// struct SelectBook {
///     title: String,
/// }
/// 
/// async fn select_books(title: String, db: &sqlx::PgPool) -> Result<Vec<Book>> {
///     let res = Book::select(&SelectBook {
///         title: title,
///     })
///     .fetch_all(db)
///     .await?;
///     Ok(res)
/// }
/// ```
/// Compilation fails because select is not yet implemented.
///  
/// <br>
/// <br>
/// 
/// ### Attribute Definition&ensp;<sub>(see [Attribute Notation](derive@Table#attribute-notation) and [Attribute Documentation](derive@Table#attribute-documentation))</sub>
/// 
/// ##### Struct Attributes:
/// `$[sqly((`[`table`](derive@Table#table)`)! (= `[`Path`](derive@Table#table)`)!) // required`<br>
/// `$[sqly((`[`table_name`](derive@Table#table)`)! (= `[`String`](derive@Table#table)`)!) // currently required, soon obsolete`<br>
/// `$[sqly((`[`rename`](derive@Table#rename)`)? (= `[`String`](derive@Table#rename)`)!)`<br>
/// 
/// `$[sqly((`[`print`](derive@Table#dev-attributes)`)?)`<br>
/// `$[sqly((`[`debug`](derive@Table#dev-attributes)`)?)`<br>
/// 
/// ##### Field Attributes:
/// `$[sqly((`[`column`](derive@Table#column)`)? (= `[`String`](derive@Table#column)`)!)`<br>
/// `$[sqly((`[`rename`](derive@Table#rename)`)? (= `[`String`](derive@Table#rename)`)!)`<br>
/// 
/// `$[sqly((`[`skip`](derive@Table#skip)`)?)`
pub use sqly_macros::Select;
