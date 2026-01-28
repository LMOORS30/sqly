use sqly::derive::*;

/*

In some cases sqlx type checking does not work as desired.
#[sqly(infer)] can be used to disable this behavior for
individual fields, this is required for custom sqlx types.

This is a shorthand for the type overrides available on
#[sqly(column)] and #[sqly(value)], it does not have any
effect on the queries or code executed at runtime.

*/

#[derive(Delete)]
#[sqly(table = "c16")]
struct C16Override {
    // this
    #[sqly(infer)]
    one: C16SqlxType,

    // is equivalent to this
    #[sqly(value = self.two as _)]
    #[sqly(column = "two: _")]
    two: C16SqlxType,
}

#[derive(sqly::sqlx::Type)]
#[sqlx(type_name = "db_enum")]
enum C16SqlxType {
    One,
    Two,
}

#[test]
fn c16_override_type() {
    let obj = C16Override {
        one: C16SqlxType::One,
        two: C16SqlxType::Two,
    };

    let (sql, args) = obj.delete_sql();

    assert_eq!(
        sql,
        r#"
DELETE FROM c16 AS "self"
WHERE
	"one" = $1 AND
	"two" = $2
	"#
        .trim_ascii()
    )
}
