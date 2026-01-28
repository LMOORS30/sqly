use anyhow::Result;
use sqly::derive::*;

/*

SQL queries can be generated with their respective derives,
they are formed based on the fields and attributes of the
derived struct.

As with all other derives from now on, the generated query
is checked at compile time by sqlx macros, as long as the
default "checked" feature has not been disabled.

*/

#[derive(Delete)]
#[sqly(table = "c11")]
struct C11Delete {
    id: i32,
}

async fn c11_delete(id: i32, db: &sqlx::postgres::PgPool) -> Result<bool> {
    let res = C11Delete { id }.delete().execute(db).await?;
    let deleted = res.rows_affected() > 0;
    Ok(deleted)
}

/*

The fields represent the WHERE clause of the DELETE query.
The names of the fields will be used as the names of the
database columns. Columns will be evaluated to equal the
value bound by their field.

Attributes shown later can be used to customize this
behavior, but in many cases the defaults should be
sufficient.

*/

#[test]
fn c11_derive_delete() {
    let obj = C11Delete { id: 1 };

    // the sqlx query can be built direcly as follows
    let query: sqlx::query::Query<_, _> = obj.delete();

    // or its raw parts can be returned first
    let (sql, args) = obj.delete_sql();

    // the arguments store `obj.id` as `$1` in the query
    let _: &Result<sqlx::postgres::PgArguments, _> = &args;

    // while the SQL was compiled as a static string
    assert_eq!(
        sql,
        r#"
DELETE FROM c11 AS "self"
WHERE
	"id" = $1
	"#
        .trim_ascii()
    );

    // the sqlx query can also be built from its parts
    let query = C11Delete::delete_from((sql, args));
}
