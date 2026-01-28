use anyhow::Result;
use sqly::derive::*;

/*

SQL INSERT works very similar to DELETE, except the fields
represent the column list and the VALUES clause instead.

Fields can have #[sqly(skip)] applied, excluding them from
the generated query and bound arguments. The derive will
behave as if the skipped fields do not exist.

Most attributes no longer have any effect when applied to a
skipped field. Some might raise an error as to prevent any
ambiguity, others might be content as a no-op.

*/

#[derive(Insert)]
#[sqly(table = "c12")]
struct C12Insert {
    value: &'static str,
    #[sqly(skip)]
    skipped: usize,
    other: String,
}

async fn c12_insert(obj: &C12Insert, db: &sqlx::postgres::PgPool) -> Result<()> {
    obj.insert().execute(db).await?;
    Ok(())
}

#[test]
fn c12_derive_insert() {
    let obj = C12Insert {
        value: "one",
        other: "two".into(),
        skipped: 3,
    };

    let (sql, args) = obj.insert_sql();

    assert_eq!(
        sql,
        r#"
INSERT INTO c12 AS "self" (
	"value",
	"other"
) VALUES (
	$1,
	$2
)
	"#
        .trim_ascii()
    )
}
