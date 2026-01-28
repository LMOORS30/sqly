use sqly::derive::*;

/*

#[derive(Table)] can be used to generate struct definitions
with the other query derives applied. This is often useful
to prevent many similar definitions, but never required.

#[sqly(delete, insert, select, update)] instructs the Table
derive to output definitions for the specified operations.
These will not be visible but are still available for use.

*/

#[derive(Table)]
#[sqly(table = "c34")]
#[sqly(delete, insert, select, update)]
struct C34Table {
    #[sqly(key)]
    #[sqly(skip = insert)]
    id: i32,
    one: i32,
    two: String,
    three: Option<String>,
}

#[test]
fn c34_derive_struct() {
    // delete only has the id (#[sqly(key)])
    let obj = DeleteC34Table { id: 0 };
    let (sql, args) = obj.delete_sql();
    assert_eq!(
        sql,
        r#"
DELETE FROM c34 AS "self"
WHERE
	"id" = $1
	"#
        .trim_ascii()
    );

    // insert is missing the id (#[sqly(skip = insert)])
    let obj = InsertC34Table {
        one: 1,
        two: String::from("two"),
        three: None,
    };
    let (sql, args) = obj.insert_sql();
    assert_eq!(
        sql,
        r#"
INSERT INTO c34 AS "self" (
	"one",
	"two",
	"three"
) VALUES (
	$1,
	$2,
	$3
)
	"#
        .trim_ascii()
    );

    // select filters on the id (#[sqly(key)])
    let obj = SelectC34Table { id: 0 };
    let (sql, args) = obj.select_sql();
    assert_eq!(
        sql,
        r#"
SELECT
	"self"."id",
	"self"."one",
	"self"."two",
	"self"."three"
FROM c34 AS "self"
WHERE
	"self"."id" = $1
	"#
        .trim_ascii()
    );

    // update filters on the id (#[sqly(key)])
    let obj = UpdateC34Table {
        id: 0,
        one: 1,
        two: String::from("two"),
        three: None,
    };
    let (sql, args) = obj.update_sql();
    assert_eq!(
        sql,
        r#"
UPDATE c34 AS "self"
SET
	"one" = $1,
	"two" = $2,
	"three" = $3
WHERE
	"id" = $4
	"#
        .trim_ascii()
    );
}
