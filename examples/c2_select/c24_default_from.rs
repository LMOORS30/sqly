use sqly::derive::*;

/*

To customize decoding tables from database rows the
#[sqly(from, try_from, default)] attributes are provided,
these will be used in the generated sqlx::FromRow
implementation.

*/

pub use std::borrow::Cow;

#[derive(Select)]
#[sqly(table = C24Table)]
struct C24Select {
    id: i32,
}

#[derive(Table, PartialEq, Debug)]
#[sqly(flat, try_from_flat)]
#[sqly(table = "c24", from_row)]
struct C24Table {
    id: i32,

    #[sqly(default)]
    count: i32, // the count returned from the database, 0 if null

    #[sqly(default = unknown_name())] // expression instead of Default
    #[sqly(from = String)] // decode into String, then call Cow::from(string)
    name: Cow<'static, str>, // the name returned from the database, "UNKNOWN" if null

    #[sqly(try_from = Option<String>)]
    email: Email, // the email returned from the database, fallibly converted into our custom type

    #[sqly(skip)]
    empty: Vec<()>, // always initialized as Vec::default()

    #[sqly(skip, default = vec![3, 3, 3])]
    three: Vec<i32>, // always initialized as vec![3, 3, 3]
}

pub fn unknown_name() -> Cow<'static, str> {
    Cow::Borrowed("UNKNOWN")
}

#[derive(Debug, PartialEq)]
pub struct Email(pub String);

#[derive(Debug)]
pub struct EmailError(&'static str);

impl TryFrom<Option<String>> for Email {
    type Error = EmailError;
    fn try_from(email: Option<String>) -> Result<Self, Self::Error> {
        match email {
            None => Err(EmailError("missing email")),
            Some(email) => {
                if email.is_empty() {
                    Err(EmailError("empty email"))
                } else if !email.contains('@') {
                    Err(EmailError("missing @"))
                } else {
                    Ok(Email(email))
                }
            }
        }
    }
}

// The generated FromRow definition looks something like this:
/**
```
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for C24Table {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        use ::sqly::sqlx::Row as _;
        Ok(C24Table {
            id: row.try_get("id")?,
            count: match row.try_get("count")? {
                Some(val) => val,
                None => Default::default(),
            },
            name: match row.try_get("name")? {
                Some(val) => Cow::from(val),
                None => unknown_name(),
            },
            email: Email::try_from(row.try_get("email")?)?,
            empty: Default::default(),
            three: vec![3, 3, 3],
        })
    }
}
```
*/
mod c24 {}

#[test]
fn c24_default_from() {
    let obj = C24Select { id: 4 };

    let (sql, args) = obj.select_sql();

    assert_eq!(
        sql,
        r#"
SELECT
	"self"."id",
	"self"."count",
	"self"."name",
	"self"."email"
FROM c24 AS "self"
WHERE
	"self"."id" = $1
	"#
        .trim_ascii()
    )
}
