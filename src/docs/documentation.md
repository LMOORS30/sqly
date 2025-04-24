# Attribute Documentation

##### Struct Attributes:
`#[sqly(`[`table`](#table)`,`[`rename`](#rename)`)]`<br>
`#[sqly(`[`from_row`](#from_row)`,`[`from_flat`](#flat)`,`[`flat_row`](#flat)`)]`<br>
`#[sqly(`[`flat`](#flat)`,`[`delete`](#delete)`,`[`insert`](#insert)`,`[`select`](#select)`,`[`update`](#update)`,`[`derive`](#derive)`,`[`visibility`](#visibility)`)]`<br>
`#[sqly(`[`dynamic`](#dynamic)`,`[`optional`](#optional)`,`[`serde_double_option`](#serde_double_option)`,`[`filter`](#filter)`,`[`returning`](#returning)`)]`<br>
`#[sqly(`[`unchecked`](#codegen)`,`[`crate`](#codegen)`,`[`print`](#development)`,`[`debug`](#development)`)]`<br>

##### Field Attributes:
`#[sqly(`[`column`](#column)`,`[`rename`](#rename)`)]`<br>
`#[sqly(`[`select`](#select-1)`,`[`insert`](#insert-1)`,`[`update`](#update-1)`)]`<br>
`#[sqly(`[`optional`](#optional)`,`[`filter`](#filter)`,`[`value`](#value)`,`[`infer`](#infer)`)]`<br>
`#[sqly(`[`foreign`](#foreign)`,`[`target`](#target)`,`[`named`](#named)`,`[`typed`](#typed)`)]`<br>
`#[sqly(`[`default`](#default)`,`[`from`](#from)`)]`<br>
`#[sqly(`[`skip`](#skip)`,`[`key`](#key)`)]`<br>


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

The table name will NOT be enclosed in quotes, allowing schemas and even function calls.

Invalid identifiers must be manually quoted, an alias must not be provided, it will be automatically generated.

----
```
# mod path {
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# pub struct Type;
# }
# #[derive(sqly::Delete)]
#[sqly(table = path::Type)]
# struct T { t: i32 }
```
The path to the type representing the table to be operated on.

This type is required to have [`#[derive(Table)]`](derive@Table).

<br>

#### from_row
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(from_row)]
# struct T;
```
Implements [`sqlx::FromRow`] for the table.

This is the default behavior if [`#[sqly(select)]`](#select) is specified.

Required to use this table with [`#[derive(Select)]`](derive@Select) or [`#[sqly(returning)]`](#returning) queries.

<br>

#### flat
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(flat)]
# struct T;
```
Generate the flattened struct representation of this table.

This excludes all skipped fields and matches the SQL `SELECT` list.

The struct is named by `format_ident!("Flat{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(flat = Ident)]
# struct T;
```
Same as above, except the struct is named by the given `Ident`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", flat)]
#[sqly(from_flat)]
# struct T;
```
Implements [`From<Self::Flat>`](Flat::Flat) for the table.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", flat)]
#[sqly(flat_row)]
# struct T;
```
Implements [`sqlx::FromRow`] for the flattened struct.

This can also be done with [`#[sqly(flat_derive = sqlx::FromRow)]`](#derive), but this will generate `non_snake_case` warnings due to a different implementation of the function body.

<br>

#### delete
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(delete)]
# struct T { #[sqly(key)] t: i32 }
```
Generate a delete struct with [`#[derive(Delete)]`](derive@Delete) applied.

Only fields which are marked as [`#[sqly(key)]`](#key) will be included in the generated struct.

The struct is named by `format_ident!("Delete{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(delete = Ident)]
# struct T { #[sqly(key)] t: i32 }
```
Same as above, except the struct is named by the given `Ident`.

<br>

#### insert
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(insert)]
# struct T { t: i32 }
```
Generate an insert struct with [`#[derive(Insert)]`](derive@Insert) applied.

All fields which are not marked as [`#[sqly(skip)]`](#skip) will be included in the generated struct.

The struct is named by `format_ident!("Insert{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(insert = Ident)]
# struct T { t: i32 }
```
Same as above, except the struct is named by the given `Ident`.

<br>

#### select
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(select)]
# struct T;
```
Generate a select struct with [`#[derive(Select)]`](derive@Select) applied.

Only fields which are marked as [`#[sqly(key)]`](#key) will be included in the generated struct.

The struct is named by `format_ident!("Select{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(select = Ident)]
# struct T;
```
Same as above, except the struct is named by the given `Ident`.

<br>

#### update
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(update)]
# struct T { #[sqly(key)] k: i32, v: i32 }
```
Generate an update struct with [`#[derive(Update)]`](derive@Update) applied.

All fields which are not marked as [`#[sqly(skip)]`](#skip) will be included in the generated struct.

The struct is named by `format_ident!("Update{}", self.ident)`.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(update = Ident)]
# struct T { #[sqly(key)] k: i32, v: i32 }
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
# struct T { #[sqly(key)] t: i32 }
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

#### returning
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(returning)]
# struct T;
```
Returns this table through the SQL `RETURNING` clause in generated queries.

Generates the same output list and uses the same [`sqlx::FromRow`] definition as [`#[derive(Select)]`](#derive@Select).

When not applied to [`#[derive(Table)]`](derive@Table) the type specified with [`#[sqly(table)]`](#table) will be returned instead.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(returning = Table)]
# struct T;
```
Same as above, except the path to the table to be returned is specified.

This type is required to have [`#[derive(Table)]`](derive@Table).

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(returning = { field, other })]
# struct T;
```
Returns the specified fields as a scalar or tuple through the SQL `RETURNING` clause in generated queries.

The fields must match an identifier of the fields in either this struct or the type specified with [`#[sqly(table)]`](#table).

Unlike [`#[sqly(returning)]`](#returning) for tables, the [`#[sqly(default)]`](#default) and [`#[sqly(from)]`](#from) attributes are not applied.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(returning = Table { field, other })]
# struct T;
```
Same as above, except the path to the table with the fields is specified.

This type is required to have [`#[derive(Table)]`](derive@Table).

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", insert, update, from_row)]
#[sqly(insert_returning = { field })]
#[sqly(update_returning = Self)]
# struct T { #[sqly(key)] k: i32, field: i32 }
```
Same as all of the above, except only for the operations specified.

This overrides the value set with `returning`.

<br>

#### dynamic
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", optional, insert)]
#[sqly(dynamic)]
# struct T { t: i32 }
```
Queries with [`#[sqly(optional)]`](#optional) must be generated at runtime, this requires storing the SQL in an owned string and prevents the use of the [`Delete`](Delete), [`Insert`](Insert), [`Select`](Select) and [`Update`](Update) traits (the corresponding `Impl` traits must be used instead). Additionally, the implementation generated for the `Check` traits only tests for the case of all optional fields being provided, possibly allowing additional runtime query errors to occur (although this should not make any difference in practice).

[`#[sqly(dynamic)]`](#dynamic) must be used to explicitly opt-in for this behavior.

<br>

#### serde_double_option
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", optional)]
#[sqly(serde_double_option)]
# struct T;
```
Adds a [serde attribute](https://serde.rs/field-attrs.html) to every field generated with a type wrapped in two `Option`s.

```text
#[serde(default, with = "sqly::double_option", skip_serializing_if = "Option::is_none")]
```

Will likely generate errors if a [serde derive](https://serde.rs/derive.html) is not specified with [`#[sqly(query_derive)]`](#derive).

Intended but not required to be used together with [`#[sqly(optional)]`](#optional).

Only available with the [`serde`](https://github.com/LMOORS30/sqly#features) feature enabled.

See [serde_with](https://docs.rs/serde_with/3.12.0/serde_with/rust/double_option/index.html) for more information.

<br>

#### codegen
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
#[sqly(crate = ::sqly)]
# struct T;
```
Specify the path to the `sqly` crate instance to use in generated code.

<br>

#### development
---
```compile_fail
# #[derive(sqly::Insert)]
# #[sqly(table = "")]
#[sqly(print)]
# struct T { t: i32 }
```
Print generated queries as compile time errors.

---
```compile_fail
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(debug)]
# struct T;
```
Print generated rust code as compile time errors.

---
```compile_fail
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(print = panic)]
#[sqly(debug = panic)]
# struct T;
```
Same as the above, except more explicit.

---
```ignore
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(print = warn)]
#[sqly(debug = warn)]
# struct T;
```
Same as above, except as `deprecated` warnings.

---
```ignore
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(print = stdout)]
# struct T1;
# #[derive(sqly::Table)]
# #[sqly(table = "")]
#[sqly(debug = stderr)]
# struct T2;
```
Same as above, except printed directly to the specified output stream.

Use cases:&ensp;`cargo check > queries.txt`&ensp;or&ensp;`cargo check 2> generated.rs`.

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
# t: i32,
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
# t: i32,
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
# t: i32,
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
# #[derive(sqly::Insert)]
# #[sqly(table = "")]
# struct T {
#[sqly(insert = "$i")]
# t: i32,
# }
```
The SQL expression to insert the column for this field.

This [Variadic Attribute](docs::attr::note#variadic-attributes) supports [String Placeholders](docs::attr::note#string-placeholders), and they are necessary to generate valid queries.

The value bound by this field must be referenced as `$i`, values bound by other fields can be referenced by their ident.

Any field can be referenced any amount of times, including skipped fields, or not at all.

<br>

#### update
---
```
# #[derive(sqly::Update)]
# #[sqly(table = "")]
# struct T {
#[sqly(update = "$column = $i")]
# t: i32,
# #[sqly(key)] k: i32,
# }
```
The SQL assignment to update the column for this field.

This [Variadic Attribute](docs::attr::note#variadic-attributes) supports [String Placeholders](docs::attr::note#string-placeholders), and they are necessary to generate valid queries.

The value bound by this field must be referenced as `$i`, the column can optionally be referenced as `$column`, values bound by other fields can be referenced by their ident.

Any field can be referenced any amount of times, including skipped and keyed fields, or not at all.

<br>

#### filter
---
```
# #[derive(sqly::Delete)]
# #[sqly(table = "")]
# struct T {
#[sqly(filter = "$column = $i")]
# t: i32,
# }
```
The SQL condition to filter in the `WHERE` clause generated by queries.

This [Variadic Attribute](docs::attr::note#variadic-attributes) supports [String Placeholders](docs::attr::note#string-placeholders), and they are necessary to generate valid queries.

The value bound by this field must be referenced as `$i`, the column can optionally be referenced as `$column`, values bound by other fields can be referenced by their ident.

This attribute can be applied to both the struct and its fields, all of which are generated and evaluated with `AND` operators.

Any field can be referenced any amount of times, including skipped and value fields, or not at all.

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

#### optional
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", dynamic, insert)]
#[sqly(optional)]
# struct T {
# t: i32,
# }
```
Declares a field or set of fields as optional.

An optional field will only be included in the generated query if its runtime value resolves to an `Option::Some`, otherwise it will behave as if it was skipped. Will generate type errors if the value bound does not resolve to an `Option`. Does not affect the SQL `SELECT` list, [`sqlx::FromRow`] definition and [`Flat`](#flat) struct, as these do not involve runtime values before execution.

Any field can be optional, even those with [`#[sqly(value)]`](#value) or [`#[sqly(filter)]`](#filter), among others. The value to be bound by this field determines whether the part of the query relevant to this field is included. This check happens regardless of whether the field binds its own value (e.g. [`#[sqly(insert = "default")]`](#insert-1)) or others (e.g. [`#[sqly(update = "c=COALESCE($i,$j)")]`](#update-1)).

This attribute can be applied to both the struct and its fields. When applied to the struct its behavior is different depening on the derive. [`#[derive(Table)]`](derive@Table) will set all fields as optional, additionally, it will wrap all optional fields in an `Option` when included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs. Other derives will only set fields whose type is already wrapped in an `Option` as optional, but fields can be individually specified as optional regardless of their type.

[`#[sqly(dynamic)]`](#dynamic) is required to explicitly opt-in for this behavior.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", dynamic, insert)]
#[sqly(optional = keys)]
# struct T {
# #[sqly(key)]
# t: i32,
# }
```
Apply optional to all fields which are marked as a key.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", dynamic, insert)]
#[sqly(optional = values)]
# struct T {
# t: i32,
# }
```
Apply optional to all fields which are not marked as a key.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", dynamic, optional, insert)]
# struct T {
#[sqly(optional = false)]
# t: i32,
# v: i32,
# }
```
Overrides any value set on the struct and disables optional for this field.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", dynamic, insert, update)]
#[sqly(insert_optional = keys)]
# struct T {
# t: i32,
#[sqly(update_optional = false)]
# #[sqly(key)] k: i32,
# }
```
Same as all of the above, except only for the operations specified.

This overrides the value set with `optional`.

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
# t: i32,
# }
```
Disable type compatibility checking for this field, does not influence nullability checks.

This is a shorthand for the [`#[sqly(column)]`](#column) and [`#[sqly(value)]`](#value) type overrides.

The following attributes achieve the same effect:

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
# t: i32,
# }
```
Mark this field as a foreign table.

This attribute has several implications:

When generating `SELECT` queries an SQL `JOIN` expression is added for the foreign table. Additionally, all columns needed for the [`sqlx::FromRow`] implementation of the foreign struct are selected. This works recursively and for any amount of foreign tables. Joined tables and selected columns are renamed in order to avoid name conflicts.

The type of this field is required to have [`#[derive(Table)]`](derive@Table) and must be a path without any generics. The only exception is `Option<T>`, where the same restrictions apply to `T` and the identifier of `Option` must not be renamed. This prompts the generated expression to perform a `LEFT JOIN` instead of an `INNER JOIN`.

When generating [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs this field will have its ident and type changed in order to match the column used in the SQL `JOIN` expression. When generating the [`Table::Flat`](#flat) struct all fields are recursively flattened and renamed in order to match the SQL `SELECT` list.

The [`#[sqly(target)]`](#target), [`#[sqly(named)]`](#named) and [`#[sqly(typed)]`](#typed) attributes further explain this behavior.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(foreign = "$INNER JOIN other AS $other ON $other.id = $table.other_id")]
# t: i32,
# }
```
Set a custom SQL `JOIN` expression for the foreign table.

This [Variadic Attribute](docs::attr::note#variadic-attributes) supports [String Placeholders](docs::attr::note#string-placeholders), and they are necessary to generate valid queries.

The joined table must be renamed to `$other` and the current table must be referenced as `$table`.

Joins should be specified with one of `$INNER`, `$inner`, `$LEFT`, `$left` to support further `JOIN`s on this table.

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
# #[sqly(foreign)]
#[sqly(target = field)]
# t: i32,
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
# t: i32,
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
# t: i32,
# }
```
Set the ident of this field when included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs.

When used with [`#[sqly(foreign)]`](#foreign) this is supposed to match the name of the column used in the SQL `JOIN` expression.

For foreign fields a new ident is required, if not specified a default ident is constructed by a series of rules:
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
# t: i32,
# }
```
Set the type of this field when included in generated [`Delete`](derive@Delete), [`Insert`](derive@Insert), [`Select`](derive@Select) and [`Update`](derive@Update) structs.

When used with [`#[sqly(foreign)]`](#foreign) this is supposed to match the type of the column used in the SQL `JOIN` expression.

For foreign fields a new type is required, if not specified this is set to the type of the target field, otherwise an error will be raised.

Overrides any type modifications made by [`#[sqly(from)]`](#from), [`#[sqly(default)]`](#default) and [`#[sqly(optional)]`](#optional).

<br>

#### default
---
```
# #[derive(sqly::Table)]
# #[sqly(table = "")]
# struct T {
#[sqly(default)]
# t: i32,
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
# t: i32,
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
# t: i32,
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
# t: i32,
# }
```
Do not include this field when generating queries or structs.

When this table is included in any [`sqlx::FromRow`] or [`From<Self::Flat>`](Flat::Flat) definitions the type of this field must implement `Default`, or a custom [`#[sqly(default)]`](#default) function must be specified.

---
```
# #[derive(sqly::Table)]
# #[sqly(table = "", delete, insert, select, update)]
# struct T {
#[sqly(skip = from_row, delete, insert, select, update)]
# t: i32,
# #[sqly(key)] k: i32, v: i32,
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
# t: i32,
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
