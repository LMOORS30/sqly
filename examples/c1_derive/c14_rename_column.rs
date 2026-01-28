use sqly::derive::*;

/*

SQL column names are not always the best rust identifiers.
#[sqly(column)] and #[sqly(rename)] can be used to map
between the two where needed.

#[sqly(column)] sets the column to the given string,
#[sqly(rename)] applies the given naming convention.

*/

#[derive(Insert)]
#[sqly(table = "\"c~14\"")]
#[sqly(rename_all = "SCREAMING-KEBAB-CASE")]
struct C14Columns {
    full_name: &'static str,

    #[sqly(rename = "PascalCase")]
    phone_number: &'static str,

    #[sqly(column = "e@mail")]
    email_address: &'static str,

    #[sqly(rename = "none")]
    #[sqly(column = "over+RIDE")]
    value: i32,
}

#[test]
fn c14_rename_column() {
    let obj = C14Columns {
        full_name: "four",
        phone_number: "+4444",
        email_address: "4@four",
        value: 4,
    };

    let (sql, args) = obj.insert_sql();

    assert_eq!(
        sql,
        r#"
INSERT INTO "c~14" AS "self" (
	"FULL-NAME",
	"PhoneNumber",
	"E-MAIL",
	"over+RIDE"
) VALUES (
	$1,
	$2,
	$3,
	$4
)
	"#
        .trim_ascii()
    )
}
