use anyhow::Result;
use sqly::derive::*;

/*

SQL UPDATE is a bit more complicated, a distinction has to
be made between the fields used to filter and the fields
used to update.

This is done with the #[sqly(key)] attribute. Fields marked
with this attribute are known as "keys", the rest can be
referred to as "values".

Keys are used to filter in the WHERE clause of the query,
analogous to the fields in DELETE. Values are used to set
the updated columns, similar to the fields in INSERT.

*/

#[derive(Update)]
#[sqly(table = "c13")]
struct C13Update {
    #[sqly(key)]
    id1: i32,
    value_a: &'static str,

    #[sqly(key)]
    id2: i32,
    value_b: &'static str,
}

async fn c13_update(obj: &C13Update, db: &sqlx::postgres::PgPool) -> Result<u64> {
    Ok(obj.update().execute(db).await?.rows_affected())
}

#[test]
fn c13_derive_update() {
    let obj = C13Update {
        id1: 3,
        id2: 4,
        value_a: "three",
        value_b: "four",
    };

    let (sql, args) = obj.update_sql();

    assert_eq!(
        sql,
        r#"
UPDATE c13 AS "self"
SET
	"value_a" = $1,
	"value_b" = $2
WHERE
	"id1" = $3 AND
	"id2" = $4
	"#
        .trim_ascii()
    )
}
