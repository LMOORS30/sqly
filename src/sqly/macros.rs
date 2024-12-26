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
///	    SELECT
///	    	publisher.id AS publisher__id,
///	    	publisher.unique_name AS publisher__unique_name,
///	    	book.id AS book__id,
///	    	book.title AS book__title,
///	    	author.id AS author__id,
///	    	author.name AS author__name
///	    FROM publications AS self
///	    LEFT JOIN publishers AS publisher ON publisher.unique_name = self.publisher_name
///	    INNER JOIN books AS book ON book.id = self.book_id
///	    INNER JOIN authors AS author ON author.id = book.author_id
///	    WHERE
///	    	self.book_id = $1
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
/// 
/// <br>
/// <br>
/// 
/// ### Attribute Definition&ensp;<sub>(see [Attribute Notation](#attribute-notation) and [Attribute Documentation](#attribute-documentation))</sub>
/// 
/// ##### Struct Attributes:
/// `#[sqly((`[`table`](#table)`)! (= `[`String`](#table)`)!)] // required`<br>
/// `#[sqly((`[`rename`](#rename)`)? (= `[`String`](#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`flat`](#flat)`)? (= `[`Ident`](#flat)`)!)]`<br>
/// `#[sqly((`[`delete`](#delete)`)? (= `[`Ident`](#delete)`)?)]`<br>
/// `#[sqly((`[`insert`](#insert)`)? (= `[`Ident`](#insert)`)?)]`<br>
/// `#[sqly((`[`select`](#select)`)? (= `[`Ident`](#select)`)?)]`<br>
/// `#[sqly((`[`update`](#update)`)? (= `[`Ident`](#update)`)?)]`<br>
/// 
/// `#[sqly((`[`flat_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// `#[sqly((`[`query_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// `#[sqly((`[`delete_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// `#[sqly((`[`insert_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// `#[sqly((`[`select_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// `#[sqly((`[`update_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// 
/// `#[sqly((`[`flat_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// `#[sqly((`[`query_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// `#[sqly((`[`delete_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// `#[sqly((`[`insert_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// `#[sqly((`[`select_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// `#[sqly((`[`update_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// 
/// `#[sqly((`[`unchecked`](#dev-attributes)`)?)]`<br>
/// `#[sqly((`[`print`](#dev-attributes)`)?)]`<br>
/// `#[sqly((`[`debug`](#dev-attributes)`)?)]`<br>
/// 
/// ##### Field Attributes:
/// `#[sqly((`[`column`](#column)`)? (= `[`String`](#column)`)!)]`<br>
/// `#[sqly((`[`rename`](#rename)`)? (= `[`String`](#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`select`](#select-1)`)* (= `[`String`](#select-1)`)+)]`<br>
/// `#[sqly((`[`value`](#value)`)? (= `[`Expr`](#value)`)!)]`<br>
/// `#[sqly((`[`infer`](#infer)`)?)]`<br>
/// 
/// `#[sqly((`[`foreign`](#foreign)`)* (= `[`String`](#foreign)`)*)]`<br>
/// `#[sqly((`[`foreign_key`](#foreign)`)? (= `[`Ident`](#foreign)` | `[`String`](#foreign)`)!)]`<br>
/// `#[sqly((`[`foreign_named`](#foreign)`)? (= `[`Ident`](#foreign)`)!)]`<br>
/// `#[sqly((`[`foreign_typed`](#foreign)`)? (= `[`Type`](#foreign)`)!)]`<br>
/// 
/// `#[sqly((`[`skip`](#skip)`)? (= `[`delete`](#skip)` | `[`insert`](#skip)` | `[`select`](#skip)` | `[`update`](#skip)` | `[`query`](#skip)`)*)]`<br>
/// `#[sqly((`[`key`](#key)`)? (= `[`delete`](#key)` | `[`select`](#key)` | `[`update`](#key)`)*)]`<br><br>
/// 
/// ### Attribute Notation
/// A definition in the form of:<br>
/// `#[sqly((`[`name`](#attribute-notation)`)? (= `[`Value`](#attribute-notation)`)?)]`<br>
/// Represents an attribute with the specified name parsing value(s) into the given type.
/// 
/// Both the name and value are surrounded by parentheses and followed by a repetition operator, these are not matched literally but instead represent how many times the item must occur:
/// 
/// ` ! `&ensp;—&ensp;exactly once (required)<br>
/// ` ? `&ensp;—&ensp;at most once (optional)<br>
/// ` + `&ensp;—&ensp;at least once (required variadic)<br>
/// ` * `&ensp;—&ensp;zero or more (optional variadic)
/// 
/// If no value is specified in the definition there cannot be any value.
/// 
/// The value must occur the specified amount of times for each occurence of the name.
/// 
/// A singular equals sign is required when the value occurs at least once, otherwise it must be omitted.
/// 
/// Multiple values are separated by a comma, a variadic item is parsed to a value if it is not immediately followed by an equals sign, otherwise it is parsed as the name of the next attribute.
/// 
/// Multiple attributes can appear in the same `#[sqly()]` clause when separated by a comma, or can be split up into separate `#[sqly()]` clauses as desired.
/// 
/// Values defined with pipes represent an enum. Quotes are not expected unless for parsing strings.
/// 
/// <br>
/// 
/// ### String Placeholders
/// The [`#[sqly(select = "")]`](#select-1) and [`#[sqly(foreign = "")]`](#foreign) attributes allow for writing arbitrary SQL strings which will appear verbatim in generated queries. This causes issues for certain parts of the query, such as table and column names, as these will be automatically renamed as needed. String placeholders can be used in order to reference these unknown values, they will be replaced by the appropriate value before being included in the generated query. 
/// 
/// String placeholders start with a `$` sign and can appear anywhere in the SQL string.
/// 
/// The dollar sign can be escaped using `$$`, which will resolve to the literal `$` without applying any placeholder rules.
/// 
/// Placeholders reference a variable using either the `$ident` or `${ident}` syntax. The `${}` syntax is necessary when a placeholder occurs immediately before another valid identifier character (e.g. `"${ident}_2"`), but otherwise identical.
/// 
/// An error will be raised for invalid placeholders (those with missing, invalid or unknown identifiers).
/// 
/// All variables are optional and can be used any amount of times.
/// 
/// The [`#[sqly(select)]`](#select-1) and [`#[sqly(foreign)]`](#foreign) sections mention which variables are available.
/// 
/// <br>
/// <br>
/// <br>
/// 
/// # Attribute Documentation
/// 
/// <br>
/// 
/// ## Struct Attributes
/// 
/// <br>
/// 
/// #### table
/// ----
/// ```
/// # #[derive(sqly::Table)]
/// #[sqly(table = "string")]
/// # struct T;
/// ```
/// The name of the database table which this type represents.
/// 
/// All uses of the table name will be enclosed in quotes.
/// 
/// ----
/// ```
/// # mod path {
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # pub struct Type;
/// # }
/// # #[derive(sqly::Delete)]
/// #[sqly(table = path::Type)]
/// # struct D { d: i32 }
/// ```
/// The path to the type representing the table to be operated on.
/// 
/// This type is required to have [`#[derive(Table)]`](derive@Table).
/// 
/// <br>
/// 
/// #### flat
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(flat = Ident)]
/// # struct T { #[sqly(key)] t: i32 };
/// ```
/// Set the name of the generated [`Table::Flat`](Table::Flat) struct to the given `Ident`.
/// 
/// If not specified the struct will be named by `format_ident!("Flat{}", self.ident)`.
/// 
/// <br>
/// 
/// #### delete
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(delete)]
/// # struct T { #[sqly(key)] t: i32 };
/// ```
/// Generate a delete struct with [`#[derive(Delete)]`](derive@Delete) applied.
/// 
/// Only fields which are marked as [`#[sqly(key)]`](#key) will be included in the generated struct.
/// 
/// The struct will be named by `format_ident!("Delete{}", self.ident)`.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(delete = Ident)]
/// # struct T { #[sqly(key)] t: i32 };
/// ```
/// Same as above, except the struct name is set to the given `Ident`.
/// 
/// <br>
/// 
/// #### insert
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(insert)]
/// # struct T { t: i32 };
/// ```
/// Generate an insert struct with [`#[derive(Insert)]`](derive@Insert) applied.
/// 
/// All fields which are not marked as [`#[sqly(skip)]`](#skip) will be included in the generated struct.
/// 
/// The struct will be named by `format_ident!("Insert{}", self.ident)`.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(insert = Ident)]
/// # struct T { t: i32 };
/// ```
/// Same as above, except the struct name is set to the given `Ident`.
/// 
/// <br>
/// 
/// #### select
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(select)]
/// # struct T { #[sqly(key)] t: i32 };
/// ```
/// Generate a select struct with [`#[derive(Select)]`](derive@Select) applied.
/// 
/// Only fields which are marked as [`#[sqly(key)]`](#key) will be included in the generated struct.
/// 
/// The struct will be named by `format_ident!("Select{}", self.ident)`.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(select = Ident)]
/// # struct T { #[sqly(key)] t: i32 };
/// ```
/// Same as above, except the struct name is set to the given `Ident`.
/// 
/// <br>
/// 
/// #### update
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(update)]
/// # struct T { #[sqly(key)] t: i32, d: i32 };
/// ```
/// Generate an update struct with [`#[derive(Update)]`](derive@Update) applied.
/// 
/// All fields which are not marked as [`#[sqly(skip)]`](#skip) will be included in the generated struct.
/// 
/// The struct will be named by `format_ident!("Update{}", self.ident)`.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(update = Ident)]
/// # struct T { #[sqly(key)] t: i32, d: i32 };
/// ```
/// Same as above, except the struct name is set to the given `Ident`.
/// 
/// <br>
/// 
/// #### derive
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(query_derive = Clone)]
/// #[sqly(query_derive = Debug, Display)]
/// # struct T;
/// ```
/// Add the provided derive macros to all generated structs.
/// 
/// Multiple instances of this attribute are joined into a single list.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "", delete, insert)]
/// # #[sqly(query_derive = Clone)]
/// #[sqly(delete_derive = Copy)]
/// #[sqly(insert_derive = PartialEq, Eq)]
/// # struct T { #[sqly(key)] t: i32 };
/// ```
/// Add the provided derive macros to the specified generated struct.
/// 
/// Multiple instances of this attribute are joined into a single list.
/// 
/// This appends to the values set with `query_derive`.
/// 
/// <br>
/// 
/// #### visibility
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(query_visibility = pub(crate))]
/// # struct T;
/// ```
/// Set the visbility of all generated structs.
/// 
/// If not specified this defaults to the visibility of the current struct.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "", delete, insert)]
/// #[sqly(flat_visibility = pub)]
/// #[sqly(delete_visibility = ,)]
/// # pub struct T { #[sqly(key)] t: i32 };
/// ```
/// Set the visbility of the specified generated struct.
/// 
/// This overrides the value set with `query_visibility`.
/// 
/// Use `visibility = ,` to set an inherited (private) visibility.
/// 
/// <br>
/// 
/// #### dev-attributes
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(unchecked)]
/// # struct T;
/// ```
/// Disable compile time checking for all queries generated by this derive.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// #[sqly(print)]
/// # struct T;
/// ```
/// Prints generated queries to stdout at compile time.
/// 
/// As this affects the build output it will likely break rust extensions and tools.
/// 
/// Intended use: `cargo check > queries.txt`.
/// 
/// Intended for development only.
/// 
/// ---
/// ```rust,ignore
/// #[sqly(debug)]
/// ```
/// Prints generated code to stdout at compile time.
/// 
/// As this affects the build output it will likely break rust extensions and tools.
/// 
/// Intended use: `cargo check > generated.rs`.
/// 
/// Intended for development only.
/// 
/// This example is not checked in order to avoid build output in tests. lol
/// 
/// <br>
/// 
/// ## Field Attributes
/// 
/// <br>
/// 
/// #### column
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// #[sqly(column = "string")]
/// # t: i32
/// # }
/// ```
/// The name of the database column which this field maps to.
/// 
/// If not specified this defaults to the name of the field.
/// 
/// All uses of the column name will be enclosed in quotes.
/// 
/// Includes support for the sqlx `?`, `!`, `: _` and `: T` [type overrides](https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns).
/// 
/// <br>
/// 
/// #### rename
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// #[sqly(rename = "UPPERCASE")]
/// # t: i32
/// # }
/// ```
/// Rename columns according to the given naming convention.
/// 
/// One of:<br>
/// `"none"`<br>
/// `"lowercase"`<br>
/// `"UPPERCASE"`<br>
/// `"camelCase"`<br>
/// `"PascalCase"`<br>
/// `"snake_case"`<br>
/// `"kebab-case"`<br>
/// `"SCREAMING_SNAKE_CASE"`<br>
/// `"SCREAMING-KEBAB-CASE"`
/// 
/// This will rename the column regardless of whether it was specified with [`#[sqly(column)]`](#column).
/// 
/// This attribute can be applied to both the struct and its fields, where the value in fields overrides the value in the struct.
/// 
/// <br>
/// 
/// #### select
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// #[sqly(select = "$table.column AS \"$alias\"")]
/// # t: i32
/// # }
/// ```
/// The SQL expression to select the column for this field.
/// 
/// This attribute supports [String Placeholders](#string-placeholders), and they are necessary to generate valid queries.
/// 
/// The column must be renamed to `"$alias"` and the table must be referenced as `$table`.
/// 
/// No other string placeholders are available, other tables can currently not be referenced.
/// 
/// <br>
/// 
/// #### value
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "", insert)]
/// # struct T {
/// #[sqly(value = obj.field)]
/// # field: i32
/// # }
/// ```
/// The expression to be used when binding this field as an argument.
/// 
/// A reference to the instance of this object is assigned to `obj`.
/// 
/// Includes support for the sqlx `as _` [type override](https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-bind-parameters-postgres-only).
/// 
/// <br>
/// 
/// #### infer
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// #[sqly(infer)]
/// # t: i32
/// # }
/// ```
/// Disable type compatibility checking for this field, does not influence nullability checks.
/// 
/// This is a shorthand for [`#[sqly(column)]`](#column) and [`#[sqly(value)]`](#value).<br>
/// The following attributes achieve the same effect:<br>
/// `#[sqly(column = "column: _")]`<br>
/// `#[sqly(value = expr as _)]`
/// 
/// <br>
/// 
/// #### foreign
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// #[sqly(foreign)]
/// # t: i32
/// # }
/// ```
/// Mark this field as a foreign table.
/// 
/// This attribute has several implications:
/// 
/// When generating `SELECT` queries an SQL `JOIN` expression is added for the foreign table. Additionally, all columns needed for the [`Table::from_row`](Table::from_row) implementation of the foreign struct are selected. This works recursively and for any amount of foreign tables. Joined tables and selected columns are renamed in order to avoid name conflicts.
/// 
/// The type of this field is required to have [`#[derive(Table)]`](derive@Table) and must be a path without any generics. The only exception is `Option<T>`, where the same restrictions apply to `T` and the identifier of `Option` must not be renamed. This prompts the generated expression to perform a `LEFT JOIN` instead of an `INNER JOIN`.
/// 
/// When generating [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs this field will have its name and type changed in order to match the foreign key used in the SQL `JOIN` expression. When generating the [`Table::Flat`](Table::Flat) struct all fields are recursively flattened and renamed in order to match the SQL `SELECT` list.
/// 
/// The other attribute definitions in this section further explain this behavior.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// # #[sqly(foreign)]
/// #[sqly(foreign_key = field)]
/// # t: i32
/// # }
/// ```
/// Specify the ident of the field in the foreign struct to perform the SQL `JOIN` on.
/// 
/// This is required to be a field which exists in the foreign struct and is not skipped for `query`.
/// 
/// The generated SQL `JOIN` will check for equality between the column of this field and the column of the chosen foreign field.
/// 
/// If not specified a default field is chosen only if there is exactly one key field in the foreign table, otherwise an error will be raised.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// # #[sqly(foreign)]
/// #[sqly(foreign_key = "column")]
/// # t: i32
/// # }
/// ```
/// Specify the column of the foreign table to perform the SQL `JOIN` on.
/// 
/// This is the same as above, except the column is specified directly and not required to exist in the foreign struct.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// # #[sqly(foreign)]
/// #[sqly(foreign_named = ident)]
/// # t: i32
/// # }
/// ```
/// Set the name of this field when included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) or [`Update`](derive@Update) structs.
/// 
/// This is supposed to match the name of the foreign key used in the SQL `JOIN` expression.
/// 
/// If not specified a default name is constructed by a series of rules:
/// 1. If [`#[sqly(column)]`](#column) is specified, the name is set by `format_ident!("{}", self.column.to_snake_case())`.
/// 2. If a chosen foreign field is found, the name is set by `format_ident!("{}_{}", self.ident, foreign.ident)`.
/// 3. Otherwise, the name is set by `format_ident!("{}_{}", self.ident, foreign_column.to_snake_case())`.
/// 
/// This can cause problems if [`#[sqly(column)]`](#column) is also not specified, as the constructed default name will now be used to determine the column name, and this must match the foreign key used in the SQL `JOIN`. If the constructed default name does not match the name of the foreign key, and no explicit column is specified, the resulting query will be wrong. This can be resolved by specifying either [`#[sqly(column)]`](#column) or [`#[sqly(foreign_named)]`](#foreign).
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// # #[sqly(foreign)]
/// #[sqly(foreign_typed = Type)]
/// # t: i32
/// # }
/// ```
/// Set the type of this field when included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) or [`Update`](derive@Update) structs.
/// 
/// This is supposed to match the type of the foreign key used in the SQL `JOIN` expression.
/// 
/// If not specified this defaults to the type of the chosen foreign field, if no matching foreign field is found an error will be raised.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// #[sqly(foreign = "$INNER JOIN other AS $other ON $other.id = $table.other_id")]
/// # t: i32
/// # }
/// ```
/// Set a custom SQL `JOIN` expression for the foreign table.
/// 
/// This attribute supports [String Placeholders](#string-placeholders), and they are necessary to generate valid queries.
/// 
/// The joined table must be renamed to `$other` and the current table must be referenced as `$table`.
/// 
/// Joins should be specified with one of `$INNER`, `$inner`, `$LEFT`, `$left` in order to support `LEFT JOIN`s on this table.
/// 
/// Other tables can be referenced by using their unique alias relative to the current scope as a variable (e.g. `$table_name`).
/// 
/// **Note**<br>
/// Foreign fields may be included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs.
/// 
/// When performing a custom SQL `JOIN` this is likely to cause errors due to incorrect default attribute values.
/// 
/// Either specify the relevant attributes ([`#[sqly(column)]`](#column), [`#[sqly(foreign_named)]`](#foreign), [`#[sqly(foreign_typed)]`](#foreign)), or exclude this field from generated structs ([`#[sqly(skip = delete, insert, select, update)]`](#skip)).
/// 
/// **Warning**<br>
/// Nullability is not checked for SQL `JOIN`s. When performing a `LEFT JOIN` be sure to set the type of this field to an `Option`.
/// 
/// 
/// <br>
/// 
/// #### skip
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// #[sqly(skip)]
/// # t: i32
/// # }
/// ```
/// Do not include this field when generating queries or structs.
/// 
/// When used in [`#[derive(Table)]`](derive@Table) the type for this field has to implement `Default`.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "", delete, insert, select, update)]
/// # struct T {
/// #[sqly(skip = delete, insert, select, update, query)]
/// # t: i32,
/// # #[sqly(key)]
/// # k: i32,
/// # v: i32,
/// # }
/// ```
/// Same as above, except only for the operations specified.
/// 
/// When `query` is skipped the type for this field has to implement `Default`.
/// 
/// <br>
/// 
/// #### key
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "")]
/// # struct T {
/// #[sqly(key)]
/// # t: i32
/// # }
/// ```
/// Mark this field as a key.
/// 
/// Keys are used when filtering by checking for equality in the SQL `WHERE` clause.
/// 
/// Different operations regard this attribute differently:
/// 
/// [`#[derive(Delete)]`](derive@Delete) consists only of keys, therefore this attribute must not be specified.
/// 
/// [`#[derive(Insert)]`](derive@Insert) has no concept of keys, therefore this attribute must not be specified.
/// 
/// [`#[derive(Select)]`](derive@Select) consists only of keys, therefore this attribute must not be specified.
/// 
/// [`#[derive(Update)]`](derive@Update) uses the key fields to filter while using the other fields to set values.
/// 
/// [`#[derive(Table)]`](derive@Table) uses the key attribute to determine which fields to include in generated structs: 
/// 
/// When generating [`#[sqly(delete)]`](#delete) and [`#[sqly(select)]`](#select) structs only key fields are included.
/// 
/// When generating [`#[sqly(update)]`](#update) structs this attribute is passed through.
/// 
/// When generating [`#[sqly(insert)]`](#insert) structs this attribute is ignored.
/// 
/// ---
/// ```
/// # #[derive(sqly::Table)]
/// # #[sqly(table = "", delete, select, update)]
/// # struct T {
/// #[sqly(key = delete, select, update)]
/// # t: i32,
/// # v: i32,
/// # }
/// ```
/// Same as above, except only for the operations specified.
pub use sqly_macros::Table;
