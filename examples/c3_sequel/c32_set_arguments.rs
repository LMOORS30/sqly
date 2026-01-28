use sqly::derive::*;

/*

Some parts of the query have to be automatically generated,
in this case argument bindings. For this purpose custom SQL
strings support the concept of string placeholders.

Further explained in the documentation, these are variables
with a dollar sign available to relevant attributes. They
will be replaced before included in the generated query.

In the example below, $column is an optional shorthand for
the column of the current field, $i the argument bound for
the current field, and $increment_amount and $equals_self
the arguments bound by their respective fields.

*/

#[derive(Insert)]
#[sqly(table = "c32")]
struct C32Insert {
    #[sqly(insert = "COALESCE($i, 'default_value')")]
    // generates:    COALESCE($1, 'default_value')
    default_column: Option<String>,
}

#[derive(Update)]
#[sqly(table = "c32")]
struct C32Update {
    #[sqly(column = "counter")]
    #[sqly(update = "$column = $column + $i")]
    // generates:    counter = counter + $1
    increment_amount: i32,

    #[sqly(update = "$column = $increment_amount")]
    // generates:    latest_increment = $1
    latest_increment: (),

    #[sqly(key)]
    #[sqly(filter = "$i = $equals_self AND")]
    // generates:    $2 = $2 AND
    #[sqly(filter = "$equals_self = $i")]
    // generates:    $2 = $2
    equals_self: i32,
}

/*

String placeholders for argument bindings are special, they
not only reference the bound arguments but also determine
whether an argument gets bound at all. If never referenced
the argument will not be bound. If referenced multiple
times the argument will only be bound once.

The below expressions are identical to the default behavior
for these attributes, the same as if not specified at all:
```
#[sqly(insert = "$i")]
#[sqly(update = "\"$column\" = $i")]
#[sqly(filter = "\"$table\".\"$column\" = $i")]
```

*/

#[test]
fn c32_set_arguments() {
    let obj = C32Insert {
        default_column: None,
    };
    let (sql, args) = obj.insert_sql();
    assert_eq!(
        sql,
        r#"
INSERT INTO c32 AS "self" (
	"default_column"
) VALUES (
	COALESCE($1, 'default_value')
)
	"#
        .trim_ascii()
    );

    let obj = C32Update {
        increment_amount: 32,
        latest_increment: (),
        equals_self: -31,
    };
    let (sql, args) = obj.update_sql();
    assert_eq!(
        sql,
        r#"
UPDATE c32 AS "self"
SET
	counter = counter + $1,
	latest_increment = $1
WHERE
	($2 = $2 AND
	$2 = $2)
	"#
        .trim_ascii()
    );
}
