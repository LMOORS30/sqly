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

Multiple attributes can appear in the same `#[sqly()]` clause when separated by a comma, or can be split up into separate `#[sqly()]` clauses as desired. This also resolves the possibly ambiguous syntax where both values and attributes are separated by a comma.

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
Variadic attributes can be specified any amount of times with any amount of values, this is intended purely for flexibility, the parameters are treated as a single list or string.

When a variadic attribute is considered as a single string the individual strings have their leading and trailing whitespace removed and are joined with newlines.
