use sqly::derive::*;

/*

The #[derive(Table)] macro is at the core of sqly, but
simply deriving it does not do much on its own. The code
below generates an empty trait implementation, as well as
a compile time check for the table definition.

Additionally, the struct is registered in the global cache
of all sqly derives, enabling it to be referenced by other
derives. This is used by several different features, but
does not yet have any effect in this example.

*/

#[derive(Table)]
#[sqly(table = "c21")]
struct C21Table;

// The generated code looks something like this:
/**
```
impl sqly::Table for C21Table {}

impl sqly::Check for C21Table {
    fn check(&self) -> ! {
        sqlx::query!( // performs the compile time check
            "SELECT
                -- the struct has no fields,
                -- so no columns are selected
            FROM c21" // ensures the "c21" table exists
        );
        panic!() // this function is not meant to be called
    }
}
```
*/
mod c21 {}
