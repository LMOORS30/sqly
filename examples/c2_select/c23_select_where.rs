use anyhow::Result;
use sqly::derive::*;

/*

The fields in the Select struct represent the WHERE clause
of the SELECT query. They function analogous to the fields
in a Delete struct, or the keys in an update struct.

The fields in the Table struct represent the SELECT list,
they are completely unrelated to the fields representing
the WHERE clause, besides appearing in the same query.

*/

#[derive(Select)]
#[sqly(table = C23Table)]
struct C23Select {
    one: &'static str,
    #[sqly(column = "id2")]
    two: i32,
}

#[derive(Table)]
#[sqly(from_row)]
#[sqly(table = "c23")]
struct C23Table {
    #[sqly(column = "id1")]
    id: i32,
    value: String,
}

async fn c23_get_one(id: i32, db: &sqlx::postgres::PgPool) -> Result<C23Table> {
    let query = C23Select {
        one: "static filter",
        two: id,
    };
    Ok(query.select().fetch_one(db).await?)
}

#[test]
fn c23_select_where() {
    let obj = C23Select {
        one: "three",
        two: 3,
    };

    let (sql, args) = obj.select_sql();

    assert_eq!(
        sql,
        r#"
SELECT
	"self"."id1" AS "id",
	"self"."value"
FROM c23 AS "self"
WHERE
	"self"."one" = $1 AND
	"self"."id2" = $2
	"#
        .trim_ascii()
    )
}
