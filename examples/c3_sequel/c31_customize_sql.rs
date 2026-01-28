use sqly::derive::*;

/*

The #[sqly(insert, update, filter)] attributes can be used
to provide custom SQL strings to be included in the query
instead of what would otherwise be automatically generated.

Fields with a static SQL query are not bound as arguments,
their value has no influence on the executed query.

*/

#[derive(Insert)]
#[sqly(table = "c31")]
struct C31Insert {
    #[sqly(insert = "DEFAULT")]
    default_column: (),
}

#[derive(Update)]
#[sqly(table = "c31")]
#[sqly(filter = "counter < max", keyless)]
struct C31Update {
    #[sqly(update = "counter = counter + 1")]
    increment: (),
}

#[test]
fn c31_customize_sql() {
    let obj = C31Insert { default_column: () };
    let sql = obj.insert_sql();
    assert_eq!(
        sql,
        r#"
INSERT INTO c31 AS "self" (
	"default_column"
) VALUES (
	DEFAULT
)
	"#
        .trim_ascii()
    );

    let obj = C31Update { increment: () };
    let sql = obj.update_sql();
    assert_eq!(
        sql,
        r#"
UPDATE c31 AS "self"
SET
	counter = counter + 1
WHERE
	(counter < max)
	"#
        .trim_ascii()
    );
}
