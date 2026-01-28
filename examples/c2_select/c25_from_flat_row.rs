use sqly::derive::*;

/*

Attributes intended for the sqlx::FromRow implementation
can be useful in other contexts as well, where decoding
from a database row might not be possible or desirable.

#[sqly(flat)] generates a struct with the exact same layout
as the intended database row, implying that it can be used
with the sqlx::query_as! macros.

#[sqly(from_flat, try_from_flat)] generates definitions to
convert from the flat struct into the table, the same way
the sqlx::FromRow implementation would have done so.

*/

// also see
use super::c24_default_from::*;

#[derive(Table, PartialEq, Debug)]
#[sqly(flat, try_from_flat)]
#[sqly(table = "c25")]
struct C25Table {
    id: i32,

    #[sqly(default)]
    count: i32,

    #[sqly(from = String, default = unknown_name())]
    name: Cow<'static, str>,

    #[sqly(try_from = Option<String>)]
    email: Email,

    #[sqly(skip)]
    empty: Vec<()>,

    #[sqly(skip, default = vec![3, 3, 3])]
    three: Vec<i32>,
}

// The generated FlatC25Table definition looks something like this:
/**
```
struct FlatC25Table {
    id: i32,
    count: Option<i32>,
    name: Option<String>,
    email: Option<String>,
}

impl TryFrom<FlatC25Table> for C25Table {
    // ...
}
```
*/
mod c25 {}

#[test]
fn c25_from_flat_row() {
    let flat = FlatC25Table {
        id: 1,
        count: None,
        name: None,
        email: Some(String::from("invalid email")),
    };
    assert!(C25Table::try_from(flat).is_err());

    let flat = FlatC25Table {
        id: 1,
        count: None,
        name: None,
        email: Some(String::from("valid@email")),
    };
    let res = C25Table {
        id: 1,
        count: 0,
        name: Cow::Borrowed("UNKNOWN"),
        email: Email(String::from("valid@email")),
        empty: Vec::new(),
        three: vec![3, 3, 3],
    };
    assert_eq!(C25Table::try_from(flat).unwrap(), res);
}
