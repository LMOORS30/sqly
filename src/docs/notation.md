### Attribute Notation
A definition in the form of:

`#[sqly((`[`name`](#)`)?)]` or<br>
`#[sqly((`[`name`](#)`)? (= `[`Value`](#)`)?)]`

Represents an attribute with the specified name, parsing value(s) into the given type.

The repetition operators represent how many times the item must occur:

` ! `&ensp;—&ensp;exactly once (required)<br>
` ? `&ensp;—&ensp;at most once (optional)<br>
` + `&ensp;—&ensp;at least once (required variadic)<br>
` * `&ensp;—&ensp;zero or more (optional variadic)

The value must occur the specified amount of times for each occurrence of the name.

Either a singular equals sign or a pair of surrounding parentheses must be used for any values.

Both values and names are separated by commas, but names can also be split across `#[sqly()]` clauses.

Values defined with pipes represent an enum. Quotes are not expected unless for parsing strings.

<br>

### String Placeholders
Some attributes allow for writing arbitrary SQL strings which will appear verbatim in generated queries. This causes issues for certain parts of the query, such as table names and parameter bindings, as these will be automatically generated as needed. String placeholders can be used in order to reference these unknown values, they will be replaced by the correct value before being included in the generated query.

String placeholders start with a `$` sign and can appear anywhere in the SQL string.

The dollar sign can be escaped using `$$`, which will resolve to the literal `$` without applying any placeholder rules.

Placeholders reference a variable using either the `$ident` or `${ident}` syntax. The `${}` syntax is necessary when a placeholder occurs immediately before another valid identifier character (e.g. `"${ident}_2"`), but otherwise identical.

An error will be raised for invalid placeholders (those with missing, invalid or unknown identifiers).

All variables are optional and can be used any amount of times.

The relevant attributes mention which variables are available to them.

<br>

### Variadic Attributes
Variadic attributes can be specified any amount of times with any amount of values, this is intended purely for flexibility, the parameters are parsed as a single list or string.

When a variadic attribute is parsed into single a string the individual strings have their leading and trailing whitespace removed and are joined with a newline and tab.
