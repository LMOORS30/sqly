use anyhow::Result;
use sqly::derive::*;

/*

The SQL RETURNING clause can be specified with the
#[sqly(returning)] attribute. Braces are used to enclose
field names which will be returned as a scalar or tuple.

*/

#[derive(Delete)]
#[sqly(table = "c26", returning = { delete_id })]
struct C26Delete {
    #[sqly(column = "id")]
    delete_id: i32,
}

async fn c26_delete(id: i32, db: &sqlx::postgres::PgPool) -> Result<Option<i32>> {
    Ok(C26Delete { delete_id: id }.delete().fetch_optional(db).await?)
}

/*

If no braces are given the query will return a whole table
instead. This is only possible if another struct with
#[derive(Table)] was specified.

*/

#[derive(Table)]
#[sqly(table = "c26", from_row)]
struct C26Table {
    id: i32,
    one: String,
    two: String,
}

#[derive(Insert)]
#[sqly(table = C26Table, returning)]
struct C26Insert {
    two: &'static str,
}

async fn c26_insert(obj: &C26Insert, db: &sqlx::postgres::PgPool) -> Result<C26Table> {
    Ok(obj.insert().fetch_one(db).await?)
}

#[test]
fn c26_sql_returning() {
    let obj = C26Delete { delete_id: 6 };
    let (sql, args) = obj.delete_sql();
    assert_eq!(
        sql,
        r#"
DELETE FROM c26 AS "self"
WHERE
	"id" = $1
RETURNING
	"self"."id" AS "delete_id"
	"#
        .trim_ascii()
    );

    let obj = C26Insert { two: "two" };
    let (sql, args) = obj.insert_sql();
    assert_eq!(
        sql,
        r#"
INSERT INTO c26 AS "self" (
	"two"
) VALUES (
	$1
)
RETURNING
	"self"."id",
	"self"."one",
	"self"."two"
	"#
        .trim_ascii()
    );
}
