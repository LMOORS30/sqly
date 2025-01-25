### Attribute Notation
A definition in the form of:<br>
`#[sqly((`[`name`](#attribute-notation)`)? (= `[`Value`](#attribute-notation)`)?)]`<br>
Represents an attribute with the specified name parsing value(s) into the given type.

Both the name and value are surrounded by parentheses and followed by a repetition operator, these are not matched literally but instead represent how many times the item must occur:

` ! `&ensp;—&ensp;exactly once (required)<br>
` ? `&ensp;—&ensp;at most once (optional)<br>
` + `&ensp;—&ensp;at least once (required variadic)<br>
` * `&ensp;—&ensp;zero or more (optional variadic)

If no value is specified in the definition there cannot be any value.

The value must occur the specified amount of times for each occurence of the name.

A singular equals sign is required when the value occurs at least once, otherwise it must be omitted.

Multiple values are separated by a comma, a variadic item is parsed to a value if it is not immediately followed by an equals sign and not an identifier while a literal is expected, otherwise it is parsed as the name of the next attribute.

Multiple attributes can appear in the same `#[sqly()]` clause when separated by a comma, or can be split up into separate `#[sqly()]` clauses as desired, this also resolves the ambiguous syntax where both values and attributes are separated by a comma.

Values defined with pipes represent an enum. Quotes are not expected unless for parsing strings.

<br>

### String Placeholders
The [`#[sqly(select = "")]`](docs::attr#select-1) and [`#[sqly(foreign = "")]`](docs::attr#foreign) attributes allow for writing arbitrary SQL strings which will appear verbatim in generated queries. This causes issues for certain parts of the query, such as table and column names, as these will be automatically renamed as needed. String placeholders can be used in order to reference these unknown values, they will be replaced by the appropriate value before being included in the generated query. 

String placeholders start with a `$` sign and can appear anywhere in the SQL string.

The dollar sign can be escaped using `$$`, which will resolve to the literal `$` without applying any placeholder rules.

Placeholders reference a variable using either the `$ident` or `${ident}` syntax. The `${}` syntax is necessary when a placeholder occurs immediately before another valid identifier character (e.g. `"${ident}_2"`), but otherwise identical.

An error will be raised for invalid placeholders (those with missing, invalid or unknown identifiers).

All variables are optional and can be used any amount of times.

The [`#[sqly(select)]`](docs::attr#select-1) and [`#[sqly(foreign)]`](docs::attr#foreign) sections mention which variables are available.
