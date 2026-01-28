use anyhow::Result;
use sqly::derive::*;

/*

SQL SELECT is more complicated, unlike the other queries it
requires the Table derive, this will be responsible for the
majory of the query generation.

Instead of the ability to refer to tables as strings with
#[sqly(table = "c22")], the path to another struct must be
passed instead. This struct must have #[derive(Table)]
applied.

An empty struct can be used to generate the SQL SELECT list
for the given table. The table requires #[sqly(from_row)],
which generates the sqlx::FromRow definition needed to
construct the table from the results of the query.

*/

#[derive(Table)]
#[sqly(table = "c22", from_row)]
struct C22Table {
    value: i32,
}

#[derive(Select)]
#[sqly(table = C22Table)]
struct C22Select;

async fn c22_get_all(db: &sqlx::postgres::PgPool) -> Result<Vec<C22Table>> {
    Ok(C22Select.select().fetch_all(db).await?)
}

#[test]
fn c22_derive_select() {
    let sql = C22Select.select_sql();

    assert_eq!(
        sql,
        r#"
SELECT
	"self"."value"
FROM c22 AS "self"
	"#
        .trim_ascii()
    )
}
