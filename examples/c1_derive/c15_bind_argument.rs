use sqly::derive::*;

/*

By default fields simply bind their value by reference.
#[sqly(value)] can be used to pass a custom expression
instead.

Any rust expression is accepted, invalid code will simply
end up triggering compiler errors. Handling Results and
Futures is not supported (except for panics and blocks).

*/

#[derive(Update)]
#[sqly(table = "c15")]
struct C15Arguments {
    #[sqly(key)]
    #[sqly(value = {
		let id = self.id.get();
		i32::try_from(id).ok()
	})]
    id: std::num::NonZero<u64>,

    #[sqly(value = self.absolute.abs())]
    absolute: i32,

    #[sqly(value = "CONST")]
    constant: (),

    // required for sqlx compile time parameter type checking
    #[sqly(value = &self.list[..])]
    list: Vec<i32>,
}

#[test]
fn c15_bind_argument() {
    let obj = C15Arguments {
        // binds as none if too large for an i32
        id: std::num::NonZero::new(5).expect("5"),
        absolute: -5, // binds as (-5).abs()
        constant: (), // binds as "CONST"
        list: vec![5, 4, 3, 2, 1],
    };

    let (sql, args) = obj.update_sql();

    assert_eq!(
        sql,
        r#"
UPDATE c15 AS "self"
SET
	"absolute" = $1,
	"constant" = $2,
	"list" = $3
WHERE
	"id" = $4
	"#
        .trim_ascii()
    )
}
