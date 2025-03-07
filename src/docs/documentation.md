# Attribute Documentation

<br>

## Struct Attributes

<br>

#### table
----
```
# #[derive(sqly::Table)]
#[sqly(table = "string")]
# struct T;
```
The name of the database table which this type represents.

All uses of the table name will be enclosed in quotes.

----
```
# mod path {
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# pub struct Type;
# }
# #[derive(sqly::Delete)]
#[sqly(table = path::Type)]
# struct D { d: i32 }
```
The path to the type representing the table to be operated on.

This type is required to have [`#[derive(Table)]`](derive@Table).

<br>

#### flat
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(flat)]
# struct T { #[sqly(key)] t: i32 };
```
Generate the flattened struct representation of this table.

This excludes all skipped fields and matches the SQL `SELECT` list.

Implements the [`sqly::Flat`](Flat), [`sqlx::FromRow`] and `From<Flat>` traits.

The struct is named by `format_ident!("Flat{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(flat = Ident)]
# struct T { #[sqly(key)] t: i32 };
```
Same as above, except the struct is named by the given `Ident`.

<br>

#### delete
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(delete)]
# struct T { #[sqly(key)] t: i32 };
```
Generate a delete struct with [`#[derive(Delete)]`](derive@Delete) applied.

Only fields which are marked as [`#[sqly(key)]`](#key) will be included in the generated struct.

The struct is named by `format_ident!("Delete{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(delete = Ident)]
# struct T { #[sqly(key)] t: i32 };
```
Same as above, except the struct is named by the given `Ident`.

<br>

#### insert
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(insert)]
# struct T { t: i32 };
```
Generate an insert struct with [`#[derive(Insert)]`](derive@Insert) applied.

All fields which are not marked as [`#[sqly(skip)]`](#skip) will be included in the generated struct.

The struct is named by `format_ident!("Insert{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(insert = Ident)]
# struct T { t: i32 };
```
Same as above, except the struct is named by the given `Ident`.

<br>

#### select
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(select)]
# struct T { #[sqly(key)] t: i32 };
```
Generate a select struct with [`#[derive(Select)]`](derive@Select) applied.

Only fields which are marked as [`#[sqly(key)]`](#key) will be included in the generated struct.

The struct is named by `format_ident!("Select{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(select = Ident)]
# struct T { #[sqly(key)] t: i32 };
```
Same as above, except the struct is named by the given `Ident`.

<br>

#### update
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(update)]
# struct T { #[sqly(key)] t: i32, d: i32 };
```
Generate an update struct with [`#[derive(Update)]`](derive@Update) applied.

All fields which are not marked as [`#[sqly(skip)]`](#skip) will be included in the generated struct.

The struct is named by `format_ident!("Update{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(update = Ident)]
# struct T { #[sqly(key)] t: i32, d: i32 };
```
Same as above, except the struct is named by the given `Ident`.

<br>

#### derive
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(query_derive = Clone)]
#[sqly(query_derive = Debug, Display)]
# struct T;
```
Add the provided derive macros to all generated query structs.

Multiple instances of this attribute are joined into a single list.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", delete, insert)]
# #[sqly(query_derive = Clone)]
#[sqly(delete_derive = Copy)]
#[sqly(insert_derive = PartialEq, Eq)]
# struct T { #[sqly(key)] t: i32 };
```
Add the provided derive macros to the specified generated struct.

Multiple instances of this attribute are joined into a single list.

This appends to the values set with `query_derive`.

<br>

#### visibility
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(query_visibility = pub(crate))]
# struct T;
```
Set the visbility of all generated query structs.

If not specified this defaults to the visibility of the current struct.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", delete, insert)]
#[sqly(flat_visibility = pub)]
#[sqly(delete_visibility = ,)]
# pub struct T { #[sqly(key)] t: i32 };
```
Set the visbility of the specified generated struct.

This overrides the value set with `query_visibility`.

Use `visibility = ,` to set an inherited (private) visibility.

<br>

#### dev-attributes
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(crate = ::sqly)]
# struct T;
```
Specify the path to the `sqly` crate instance to use in generated code.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(unchecked)]
# struct T;
```
Disable compile time checking for all queries generated by this derive.

This has no effect if the default [`checked`](https://github.com/LMOORS30/sqly#features) feature is not enabled.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(print)]
# struct T;
```
Prints generated queries to stdout at compile time.

Intended use: `cargo check > queries.txt`.

Intended for development only.

---
```rust,ignore
#[sqly(debug)]
```
Prints generated code to stdout at compile time.

Intended use: `cargo check > generated.rs`.

Intended for development only.

This example is not tested in order to avoid build output in tests. lol

<br>
<br>
<br>

## Field Attributes

<br>

#### column
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(column = "string")]
# t: i32
# }
```
The name of the database column which this field maps to.

If not specified this defaults to the ident of the field.

All uses of the column name will be enclosed in quotes.

Includes support for the sqlx&ensp;`?`&ensp;`!`&ensp;`: _`&ensp;and&ensp;`: T`&ensp;[type overrides](sqlx::query!#type-overrides-output-columns).

<br>

#### rename
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(rename = "UPPERCASE")]
# t: i32
# }
```
Rename columns according to the given naming convention.

One of:<br>
`"none"`<br>
`"lowercase"`<br>
`"UPPERCASE"`<br>
`"camelCase"`<br>
`"PascalCase"`<br>
`"snake_case"`<br>
`"kebab-case"`<br>
`"SCREAMING_SNAKE_CASE"`<br>
`"SCREAMING-KEBAB-CASE"`

This will rename the column regardless of whether it was specified with [`#[sqly(column)]`](#column).

This attribute can be applied to both the struct and its fields, where the value in fields overrides the value in the struct.

<br>

#### select
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(select = "$table.$column")]
# t: i32
# }
```
The SQL expression to select the column for this field.

This [Variadic Attribute](docs::attr::note#variadic-attributes) supports [String Placeholders](docs::attr::note#string-placeholders), and they are necessary to generate valid queries.

The table must be referenced as `$table`, the column can optionally be referenced as `$column`, the alias must not be set.

No other string placeholders are available, other tables and parameter bindings can currently not be referenced.

<br>

#### insert
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", insert)]
# struct T {
#[sqly(insert = "$i")]
# t: i32
# }
```
The SQL expression to insert the column for this field.

This [Variadic Attribute](docs::attr::note#variadic-attributes) supports [String Placeholders](docs::attr::note#string-placeholders), and they are necessary to generate valid queries.

The value bound by this field must be referenced as `$i`, values bound by other fields can be referenced by their name.

Any field can be referenced any amount of times, including skipped fields, or not at all.

<br>

#### update
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", update)]
# struct T {
#[sqly(update = "$i")]
# t: i32,
# #[sqly(key)] d: i32
# }
```
The SQL expression to update the column for this field.

This [Variadic Attribute](docs::attr::note#variadic-attributes) supports [String Placeholders](docs::attr::note#string-placeholders), and they are necessary to generate valid queries.

The value bound by this field must be referenced as `$i`, the column can optionally be referenced as `$column`, values bound by other fields can be referenced by their name.

Any field can be referenced any amount of times, including skipped and keyed fields, or not at all.

<br>

#### filter
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct Table;
# #[derive(sqly::Select)]
# #[sqly(table = Table)]
# struct T {
#[sqly(filter = "$column = $i")]
# t: i32,
# }
```
The SQL condition to filter in the `WHERE` clause generated by queries.

This [Variadic Attribute](docs::attr::note#variadic-attributes) supports [String Placeholders](docs::attr::note#string-placeholders), and they are necessary to generate valid queries.

The value bound by this field must be referenced as `$i`, the column can optionally be referenced as `$column`, values bound by other fields can be referenced by their name.

This attribute can be applied to both the struct and its fields, all of which are generated and evaluated with `AND` operators.

Any field can be referenced any amount of times, including skipped fields, or not at all.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", delete, select)]
# struct T {
# #[sqly(key)]
#[sqly(delete_filter = "$column = $i")]
#[sqly(select_filter = "$column = $i")]
# t: i32,
# }
```
Same as above, except only for the operations specified.

This overrides the value set with `filter`.

<br>

#### value
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", insert)]
# struct T {
#[sqly(value = self.field)]
# field: i32
# }
```
The Rust expression to bind this field as a parameter.

A reference to the instance of this object is assigned to `self`.

Includes support for the sqlx&ensp;`as _`&ensp;[type override](sqlx::query!#type-overrides-bind-parameters-postgres-only).

<br>

#### infer
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(infer)]
# t: i32
# }
```
Disable type compatibility checking for this field, does not influence nullability checks.

This is a shorthand for the [`#[sqly(column)]`](#column) and [`#[sqly(value)]`](#value) type overrides.

The following attributes achieve the same effect:<br>
`#[sqly(column = "column: _")]`<br>
`#[sqly(value = value as _)]`

<br>

#### foreign
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(foreign)]
# t: i32
# }
```
Mark this field as a foreign table.

This attribute has several implications:

When generating `SELECT` queries an SQL `JOIN` expression is added for the foreign table. Additionally, all columns needed for the [`Table::from_row`](Table::from_row) implementation of the foreign struct are selected. This works recursively and for any amount of foreign tables. Joined tables and selected columns are renamed in order to avoid name conflicts.

The type of this field is required to have [`#[derive(Table)]`](derive@Table) and must be a path without any generics. The only exception is `Option<T>`, where the same restrictions apply to `T` and the identifier of `Option` must not be renamed. This prompts the generated expression to perform a `LEFT JOIN` instead of an `INNER JOIN`.

When generating [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs this field will have its ident and type changed in order to match the column used in the SQL `JOIN` expression. When generating the [`Table::Flat`](#flat) struct all fields are recursively flattened and renamed in order to match the SQL `SELECT` list.

The [`#[sqly(target)]`](#target), [`#[sqly(named)]`](#named) and [`#[sqly(typed)]`](#typed) attributes further explain this behavior.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(foreign = "$INNER JOIN other AS $other ON $other.id = $table.other_id")]
# t: i32
# }
```
Set a custom SQL `JOIN` expression for the foreign table.

This [Variadic Attribute](docs::attr::note#variadic-attributes) supports [String Placeholders](docs::attr::note#string-placeholders), and they are necessary to generate valid queries.

The joined table must be renamed to `$other` and the current table must be referenced as `$table`.

Joins should be specified with one of `$INNER`, `$inner`, `$LEFT`, `$left` to support `LEFT JOIN`s on this table.

Other tables can be referenced by their unique alias relative to the current scope (e.g. `$table_name`).

**Note**<br>
Foreign fields may be included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs.

When performing a custom SQL `JOIN` this is likely to cause errors due to incorrect default attribute values.

Either specify the relevant attributes ([`#[sqly(column)]`](#column), [`#[sqly(named)]`](#named), [`#[sqly(typed)]`](#named)), or exclude this field from generated structs ([`#[sqly(skip = delete, insert, select, update)]`](#skip)).

**Warning**<br>
Nullability is not checked for SQL `JOIN`s. When performing a `LEFT JOIN` be sure to set the type of this field to an `Option`.

<br>

#### target
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(foreign, target = field)]
# t: i32
# }
```
Specify the ident of the field in the foreign struct to perform the SQL `JOIN` on.

The generated SQL `JOIN` will check for equality between the column of this field and the column of the chosen target field.

If not specified a default field is chosen only if there is exactly one key field in the foreign struct, otherwise an error will be raised.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
# #[sqly(foreign)]
#[sqly(target = "column")]
# t: i32
# }
```
Specify the column of the foreign table to perform the SQL `JOIN` on.

This is the same as above, except the column is specified directly and not required to exist in the foreign struct.

<br>

#### named
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
# #[sqly(foreign)]
#[sqly(named = ident)]
# t: i32
# }
```
Set the ident of this field when included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs.

When used with [`#[sqly(foreign)]`](#foreign) this is supposed to match the name of the column used in the SQL `JOIN` expression.

If not specified a default ident is constructed for foreign fields by a series of rules:
1. If [`#[sqly(column)]`](#column) is specified, the ident is set by `format_ident!("{}", self.column.to_snake_case())`.
2. If a target field is found, the ident is set by `format_ident!("{}_{}", self.ident, foreign.ident)`.
3. Otherwise, the ident is set by `format_ident!("{}_{}", self.ident, target.to_snake_case())`.

This can cause problems if [`#[sqly(column)]`](#column) is also not specified, as the constructed default ident will now be used to determine the column name, and this must match the column used in the SQL `JOIN`. If the constructed default ident does not match the name of the column, and no explicit column is specified, the resulting query will be wrong. This can be resolved by specifying either [`#[sqly(column)]`](#column) or [`#[sqly(named)]`](#named).

<br>

#### typed
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
# #[sqly(foreign)]
#[sqly(typed = Type)]
# t: i32
# }
```
Set the type of this field when included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs.

When used with [`#[sqly(foreign)]`](#foreign) this is supposed to match the type of the column used in the SQL `JOIN` expression.

For foreign fields a type is required, if not specified this defaults to the type of the target field, otherwise an error will be raised.

<br>

#### default
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(default)]
# t: i32
# }
```
Decode this field by using its `Default` implementation for `NULL` values in the query result.

This attribute can be used along with [`#[sqly(foreign)]`](#foreign), in which case a `LEFT JOIN` will be performed.

This will wrap the type of this field in an `Option` when included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(default = Default::default)]
# t: i32
# }
```
Same as above, except the default value is provided by calling the given path.

This can also be used to provide the default value for [`#[sqly(skip)]`](#skip).

<br>

#### from
---
```
# type Type = i32;
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(from = Type)]
# t: i32
# }
```
Decode this field by decoding into the given type before converting with the `From<T>` implementation.

This attribute can be used along with [`#[sqly(foreign)]`](#foreign), where the given type represents the foreign table.

This will replace the type of this field when included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs.

<br>

#### skip
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(skip)]
# t: i32
# }
```
Do not include this field when generating queries or structs.

When skipped for `query` in [`#[derive(Table)]`](derive@Table) the type of this field must implement `Default`, or a custom [`#[sqly(default)]`](#default) must be specified.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", delete, insert, select, update)]
# struct T {
#[sqly(skip = query, delete, insert, select, update)]
# t: i32,
# #[sqly(key)]
# k: i32,
# v: i32,
# }
```
Same as above, except only for the operations specified.

<br>

#### key
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(key)]
# t: i32
# }
```
Mark this field as a key.

Keys are used when filtering by checking for equality in the SQL `WHERE` clause.

Different operations regard this attribute differently:

[`#[derive(Delete)]`](derive@Delete) consists only of keys, therefore this attribute must not be specified.

[`#[derive(Insert)]`](derive@Insert) has no concept of keys, therefore this attribute must not be specified.

[`#[derive(Select)]`](derive@Select) consists only of keys, therefore this attribute must not be specified.

[`#[derive(Update)]`](derive@Update) uses the key fields to filter while using the other fields to set values.

[`#[derive(Table)]`](derive@Table) uses the key attribute to determine which fields to include in generated structs: 

When generating [`#[sqly(delete)]`](#delete) and [`#[sqly(select)]`](#select) structs only key fields are included.

When generating [`#[sqly(update)]`](#update) structs this attribute is passed through.

When generating [`#[sqly(insert)]`](#insert) structs this attribute is ignored.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", delete, select, update)]
# struct T {
#[sqly(key = delete, select, update)]
# t: i32,
# v: i32,
# }
```
Same as above, except only for the operations specified.

<br>
<br>
