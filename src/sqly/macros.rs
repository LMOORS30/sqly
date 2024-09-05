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
/// This macro can also be used to generate definitions for [`Delete`](Delete), [`Insert`](Insert), [`Select`](Select) and [`Update`](Update) structs, along with their appropriate derives. However, this functionality is strictly optional and can be substituted by manual definitions and derives.
/// 
/// See:<br>
/// [`#[derive(Delete)]`](derive@Delete)<br>
/// [`#[derive(Insert)]`](derive@Insert)<br>
/// [`#[derive(Select)]`](derive@Select)<br>
/// [`#[derive(Update)]`](derive@Update)<br>
/// 
/// <br>
/// 
/// # Example
/// ```
/// use sqly::*;
/// 
/// #[derive(Table)]
/// #[sqly(table = "books")]
/// struct Book;
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
/// `#[sqly((`[`delete`](#delete)`)? (= `[`Ident`](#delete)`)?)]`<br>
/// `#[sqly((`[`insert`](#insert)`)? (= `[`Ident`](#insert)`)?)]`<br>
/// `#[sqly((`[`select`](#select)`)? (= `[`Ident`](#select)`)?)]`<br>
/// `#[sqly((`[`update`](#update)`)? (= `[`Ident`](#update)`)?)]`<br>
/// 
/// `#[sqly((`[`query_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// `#[sqly((`[`delete_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// `#[sqly((`[`insert_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// `#[sqly((`[`select_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// `#[sqly((`[`update_derive`](#derive)`)* (= `[`Path`](#derive)`)+)]`<br>
/// 
/// `#[sqly((`[`query_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// `#[sqly((`[`delete_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// `#[sqly((`[`insert_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// `#[sqly((`[`select_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// `#[sqly((`[`update_visibility`](#visibility)`)? (= `[`Visibility`](#visibility)`)!)]`<br>
/// 
/// `#[sqly((`[`print`](#dev-attributes)`)?)`<br>
/// `#[sqly((`[`debug`](#dev-attributes)`)?)`<br>
/// 
/// ##### Field Attributes:
/// `#[sqly((`[`column`](#column)`)? (= `[`String`](#column)`)!)]`<br>
/// `#[sqly((`[`rename`](#rename)`)? (= `[`String`](#rename)`)!)]`<br>
/// 
/// `#[sqly((`[`skip`](#skip)`)? (= `[`delete`](#skip)` | `[`insert`](#skip)` | `[`select`](#skip)` | `[`update`](#skip)` | `[`query`](#skip)`)*)]`<br>
/// `#[sqly((`[`key`](#key)`)? (= `[`delete`](#key)` | `[`select`](#key)` | `[`update`](#key)`)*)]`<br><br>
/// 
/// ### Attribute Notation
/// A definition in the form of:<br>
/// `#[sqly((`[`name`](#attribute-notation)`)? (= `[`Value`](#attribute-notation)`)?)]`<br>
/// Represents an attribute with the specified name parsing value(s) into the given type.
/// 
/// Both the name and value are surrounded by parentheses and followed by a repetition operator, these are not matched literally but instead represent how many times the item must occur.
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
/// The path to a type representing the table which will be operated on.
/// 
/// This type is required to have [`#[derive(Table)]`](derive@Table).
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
/// The struct will be named by `format_ident!("Delete{}", name)`.
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
/// The struct will be named by `format_ident!("Insert{}", name)`.
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
/// The struct will be named by `format_ident!("Select{}", name)`.
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
/// The struct will be named by `format_ident!("Update{}", name)`.
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
/// Multiple instances of this attribute will be joined into a single list.
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
/// Multiple instances of this attribute will be joined into a single list.
/// 
/// Attributes specified with `query_derive` are not overriden but appended to.
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
/// #[sqly(insert_visibility = pub)]
/// #[sqly(delete_visibility = ,)]
/// # pub struct T { #[sqly(key)] t: i32 };
/// ```
/// Set the visbility of the specified generated struct.
/// 
/// Overrides the value set with `query_visibility` if specified.
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
/// `"lowerCamelCase"`<br>
/// `"UpperCamelCase"`<br>
/// `"snake_case"`<br>
/// `"kebab-case"`<br>
/// `"SCREAMING_SNAKE_CASE"`<br>
/// `"SCREAMING-KEBAB-CASE"`
/// 
/// This will rename the column regardless of whether it was specified with `#[sqly(column)]`.
/// 
/// This attribute can be applied to both the struct and its fields, where the value in fields overrides the value in the struct.
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
/// Do not include the field when generating queries or structs.
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
/// #[sqly(key)]
/// # k: i32,
/// # v: i32,
/// # }
/// ```
/// Same as above, except only for the operations specified.
/// 
/// The `query` option refers to the columns queried from the database. When this is skipped no column will be fetched for the given field and it will instead be initialized through the `Default` trait.
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
/// Mark the field as a key.
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
/// [`#[derive(Update)]`](derive@Update) expects at least one key field and one non key field.
/// 
/// [`#[derive(Table)]`](derive@Table) uses key fields to determine which columns to include in the generated structs: 
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
