#[allow(unused_imports)]
use super::*;

mod delete;
mod insert;
mod select;
mod update;

pub use delete::*;
pub use insert::*;
pub use select::*;
pub use update::*;



/// Applied to types which are defined to represent a database table.
/// 
/// Implements [`Table`](Table).
/// 
/// <br>
/// 
/// This macro can also be used to generate definitions for [`Delete`](Delete), [`Insert`](Insert), [`Select`](Select) and [`Update`](Update) structs along with their appropriate derives. However, this functionality is strictly optional and can be substituted by manual definitions and derives.
/// 
/// See:<br>
/// [`#[derive(Delete)]`](derive@Delete)<br>
/// [`#[derive(Insert)]`](derive@Insert)<br>
/// [`#[derive(Select)]`](derive@Select)<br>
/// [`#[derive(Update)]`](derive@Update)
/// 
/// <br>
/// 
/// ### Attribute Definition&ensp;<sub>(see [Attribute Notation](docs::attr::note) and [Attribute Documentation](docs::attr))</sub>
/// 
/// ##### Struct Attributes:
/// `#[sqly((`[`table`](docs::attr#table)`)! (= `[`String`](docs::attr#table)`)!)] // required`<br>
/// `#[sqly((`[`rename`](docs::attr#rename)`)? (= `[`String`](docs::attr#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`flat`](docs::attr#flat)`)? (= `[`Ident`](docs::attr#flat)`)!)]`<br>
/// `#[sqly((`[`delete`](docs::attr#delete)`)? (= `[`Ident`](docs::attr#delete)`)?)]`<br>
/// `#[sqly((`[`insert`](docs::attr#insert)`)? (= `[`Ident`](docs::attr#insert)`)?)]`<br>
/// `#[sqly((`[`select`](docs::attr#select)`)? (= `[`Ident`](docs::attr#select)`)?)]`<br>
/// `#[sqly((`[`update`](docs::attr#update)`)? (= `[`Ident`](docs::attr#update)`)?)]`<br>
/// 
/// `#[sqly((`[`flat_derive`](docs::attr#derive)`)* (= `[`Path`](docs::attr#derive)`)+)]`<br>
/// `#[sqly((`[`query_derive`](docs::attr#derive)`)* (= `[`Path`](docs::attr#derive)`)+)]`<br>
/// `#[sqly((`[`delete_derive`](docs::attr#derive)`)* (= `[`Path`](docs::attr#derive)`)+)]`<br>
/// `#[sqly((`[`insert_derive`](docs::attr#derive)`)* (= `[`Path`](docs::attr#derive)`)+)]`<br>
/// `#[sqly((`[`select_derive`](docs::attr#derive)`)* (= `[`Path`](docs::attr#derive)`)+)]`<br>
/// `#[sqly((`[`update_derive`](docs::attr#derive)`)* (= `[`Path`](docs::attr#derive)`)+)]`<br>
/// 
/// `#[sqly((`[`flat_visibility`](docs::attr#visibility)`)? (= `[`Visibility`](docs::attr#visibility)`)!)]`<br>
/// `#[sqly((`[`query_visibility`](docs::attr#visibility)`)? (= `[`Visibility`](docs::attr#visibility)`)!)]`<br>
/// `#[sqly((`[`delete_visibility`](docs::attr#visibility)`)? (= `[`Visibility`](docs::attr#visibility)`)!)]`<br>
/// `#[sqly((`[`insert_visibility`](docs::attr#visibility)`)? (= `[`Visibility`](docs::attr#visibility)`)!)]`<br>
/// `#[sqly((`[`select_visibility`](docs::attr#visibility)`)? (= `[`Visibility`](docs::attr#visibility)`)!)]`<br>
/// `#[sqly((`[`update_visibility`](docs::attr#visibility)`)? (= `[`Visibility`](docs::attr#visibility)`)!)]`<br>
/// 
/// `#[sqly((`[`unchecked`](docs::attr#dev-attributes)`)?)]`<br>
/// `#[sqly((`[`print`](docs::attr#dev-attributes)`)?)]`<br>
/// `#[sqly((`[`debug`](docs::attr#dev-attributes)`)?)]`<br>
/// 
/// ##### Field Attributes:
/// `#[sqly((`[`column`](docs::attr#column)`)? (= `[`String`](docs::attr#column)`)!)]`<br>
/// `#[sqly((`[`rename`](docs::attr#rename)`)? (= `[`String`](docs::attr#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`select`](docs::attr#select-1)`)* (= `[`String`](docs::attr#select-1)`)+)]`<br>
/// `#[sqly((`[`insert`](docs::attr#insert-1)`)* (= `[`String`](docs::attr#insert-1)`)+)]`<br>
/// `#[sqly((`[`update`](docs::attr#update-1)`)* (= `[`String`](docs::attr#update-1)`)+)]`<br>
/// `#[sqly((`[`value`](docs::attr#value)`)? (= `[`Expr`](docs::attr#value)`)!)]`<br>
/// `#[sqly((`[`infer`](docs::attr#infer)`)?)]`<br>
/// 
/// `#[sqly((`[`foreign`](docs::attr#foreign)`)* (= `[`String`](docs::attr#foreign)`)*)]`<br>
/// `#[sqly((`[`foreign_key`](docs::attr#foreign)`)? (= `[`Ident`](docs::attr#foreign)` | `[`String`](docs::attr#foreign)`)!)]`<br>
/// `#[sqly((`[`foreign_named`](docs::attr#foreign)`)? (= `[`Ident`](docs::attr#foreign)`)!)]`<br>
/// `#[sqly((`[`foreign_typed`](docs::attr#foreign)`)? (= `[`Type`](docs::attr#foreign)`)!)]`<br>
/// 
/// `#[sqly((`[`default`](docs::attr#default)`)? (= `[`Path`](docs::attr#default)`)?)]`<br>
/// `#[sqly((`[`from`](docs::attr#from)`)? (= `[`Type`](docs::attr#from)`)!)]`<br>
/// 
/// `#[sqly((`[`skip`](docs::attr#skip)`)? (= `[`delete`](docs::attr#skip)` | `[`insert`](docs::attr#skip)` | `[`select`](docs::attr#skip)` | `[`update`](docs::attr#skip)` | `[`query`](docs::attr#skip)`)*)]`<br>
/// `#[sqly((`[`key`](docs::attr#key)`)? (= `[`delete`](docs::attr#key)` | `[`select`](docs::attr#key)` | `[`update`](docs::attr#key)`)*)]`
/// 
/// <br>
/// 
/// # Example
/// ```
/// use sqly::*; // traits
/// # struct Page;
/// # type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
/// 
/// #[derive(Table)]
/// #[sqly(table = "books")]
/// struct Book {
///     id: i32, // #[sqly(key)] is not required
///     title: String,
///     #[sqly(foreign)] // this will perform an "INNER JOIN" on authors
///     author: Author, // the author will be selected by its #[derive(Table)] definition
///     #[sqly(skip)]
///     pages: Vec<Page>, // this will be instantiated by `Default`
/// }
/// 
/// #[derive(Table)]
/// #[sqly(table = "authors")]
/// struct Author {
///     #[sqly(key)] // mark this as the default field to perform joins on
///     id: i32,
///     name: String,
/// }
/// 
/// #[derive(Table)]
/// #[sqly(table = "publishers")]
/// struct Publisher {
///     #[sqly(key)] // default foreign key
///     id: i32,
///     unique_name: String,
/// }
/// 
/// #[derive(Table)]
/// #[sqly(table = "publications")]
/// #[sqly(select = GetBookPublications)]
/// struct Publication {
///     #[sqly(foreign)] // this will perform a "LEFT JOIN"
///     #[sqly(foreign_key = unique_name)] // use a different foreign key
///     #[sqly(column = "publisher_name")] // specify the column name for the key
///     publisher: Option<Publisher>,
///     #[sqly(key)] // include `book_id` in the select struct
///     #[sqly(foreign)] // foreign joins are recursive
///     #[sqly(foreign_key = id)] // required
///     book: Book,
/// }
/// 
/// async fn get_book_publications(book_id: i32, db: &sqlx::PgPool) -> Result<Vec<Publication>> {
///     Ok(GetBookPublications { book_id }.select().fetch_all(db).await?)
/// }
/// 
/// // executed query:
/// /*
///     SELECT
///         publisher.id AS publisher__id,
///         publisher.unique_name AS publisher__unique_name,
///         book.id AS book__id,
///         book.title AS book__title,
///         author.id AS author__id,
///         author.name AS author__name
///     FROM publications AS self
///     LEFT JOIN publishers AS publisher ON publisher.unique_name = self.publisher_name
///     INNER JOIN books AS book ON book.id = self.book_id
///     INNER JOIN authors AS author ON author.id = book.author_id
///     WHERE
///         self.book_id = $1
/// */
/// 
/// // example response:
/// /*
///     [
///         Publication {
///             publisher: Some(
///                 Publisher {
///                     id: 1,
///                     unique_name: "publisher",
///                 },
///             ),
///             book: Book {
///                 id: 1,
///                 title: "book",
///                 author: Author {
///                     id: 1,
///                     name: "author",
///                 },
///                 pages: [],
///             },
///         },
///     ]
/// */
/// ```
pub use sqly_macros::Table;
