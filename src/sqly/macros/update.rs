#[allow(unused_imports)]
use super::*;



/// Applied to types which are defined to update rows in a table.
/// 
/// <br>
/// 
/// # Example
/// ```
/// use sqly::*;
/// # type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// 
/// #[derive(Table)]
/// #[sqly(table = "books")]
/// struct Book {
///     // ...
/// }
/// 
/// #[derive(Update)]
/// #[sqly(table = Book)]
/// struct UpdateBook {
///     #[sqly(key)]
///     id: i32,
///     title: String,
/// }
/// 
/// async fn update_book(book_id: i32, title: String, db: &sqlx::PgPool) -> Result<()> {
///     Book::update(&UpdateBook {
///         id: book_id,
///         title: title,
///     })
///     .execute(db)
///     .await?;
///     Ok(())
/// }
/// ```
/// 
/// <br>
/// <br>
/// 
/// ### Attribute Definition&ensp;<sub>(see [Attribute Notation](derive@Table#attribute-notation) and [Attribute Documentation](derive@Table#attribute-documentation))</sub>
/// 
/// ##### Struct Attributes:
/// `$[sqly((`[`table`](derive@Table#table)`)! (= `[`Path`](derive@Table#table)`)!) // required`<br>
/// `$[sqly((`[`rename`](derive@Table#rename)`)? (= `[`String`](derive@Table#rename)`)!)`<br>
/// 
/// `$[sqly((`[`print`](derive@Table#dev-attributes)`)?)`<br>
/// `$[sqly((`[`debug`](derive@Table#dev-attributes)`)?)`<br>
/// 
/// ##### Field Attributes:
/// `$[sqly((`[`column`](derive@Table#column)`)? (= `[`String`](derive@Table#column)`)!)`<br>
/// `$[sqly((`[`rename`](derive@Table#rename)`)? (= `[`String`](derive@Table#rename)`)!)`<br>
/// 
/// `$[sqly((`[`skip`](derive@Table#skip)`)?)`<br>
/// `$[sqly((`[`key`](derive@Table#key)`)?)`
pub use sqly_macros::Update;
