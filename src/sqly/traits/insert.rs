#[allow(unused_imports)]
use super::*;



/// A type which can insert rows into a table.
/// 
/// Related to [`InsertCheck`](InsertCheck) and [`InsertImpl`](InsertImpl).
/// 
/// These traits are not meant to be manually implemented,
/// see [`#[derive(Insert)]`](derive@Insert)
/// or [`#[sqly(insert)]`](docs::attr#insert)
/// instead.
pub trait Insert {
    /// The type representing the table which this query will operate on.
    /// 
    /// When generated with [`#[derive(Insert)]`](derive@Insert) this type is set by the [`#[sqly(table)]`](docs::attr#table) attribute.
    /// 
    /// When generated with [`#[sqly(insert)]`](docs::attr#insert) this type is set to the struct for which [`#[derive(Table)]`](derive@Table) was called.
    /// 
    /// When this type implements the [`Table`](Table) trait the [`Table::insert`](`Table::insert`) alias is made available, this type serves no other purpose.
    type Table;

    /// The query type for the operation to be executed.
    /// 
    /// This will be equal to one of the [`sqlx::query`](mod@sqlx::query) types.
    type Query<'a>
        where Self: 'a;

    /// Returns a query which inserts rows into the table according to the definition of this type.
    /// 
    /// This function is not meant to be manualy implemented,
    /// see [`#[derive(Insert)]`](derive@Insert)
    /// or [`#[sqly(insert)]`](docs::attr#insert)
    /// instead.
    fn insert(&self) -> Self::Query<'_>;
}



/// A type which has its [`Insert`](Insert) query checked at compile time.
/// 
/// This will be generated unless
/// [`#[sqly(unchecked)]`](docs::attr#dev-attributes) is specified or
/// the default [`checked`](https://github.com/LMOORS30/sqly#features) feature is disabled.
/// 
/// This trait serves no further purpose.
pub trait InsertCheck {
    /// Calls the [`sqlx::query_as!`] macro to perform the compile time check.
    /// 
    /// This function is not meant to be called and will panic if attempted.
    fn insert_check(&self) -> !;
}



/// A type which can insert rows into a table.
/// 
/// This is the implementation generated by [`#[sqly(Insert)]`](derive@Insert).
/// 
/// An implementation for [`Insert`](Insert) will be generated as
/// `Self::`[`insert_from`](InsertImpl::insert_from)`(self.`[`insert_sql`](InsertImpl::insert_sql)`())`
/// if possible.
pub trait InsertImpl {
    /// See [`Insert::Table`](Insert::Table).
    type Table;

    /// See [`Insert::Query`](Insert::Query).
    type Query<'q, 'a>;

    /// The type from which the [`Insert::Query`](InsertImpl::Query) is built.
    /// 
    /// This will be either `&'q str` or `(&'q str, Result<`[`sqlx::Arguments`]`<'a>>)`.
    type From<'q, 'a>;

    /// The type generated by the macro derive.
    /// 
    /// This will be either `&'static str` or `(&'static str, Result<`[`sqlx::Arguments`]`<'a>>)`.
    /// 
    /// If [`#[sqly(dynamic)]`](docs::attr#dynamic) was specified this will return an owned `String` instead,
    /// possibly wrapped in an `Option` (if a complete query might not be built),
    /// and the [`Insert`](Insert) implementation will not be available
    /// (because sqlx queries cannot store owned strings),
    /// this trait must be used instead.
    type Sql<'a>
        where Self: 'a;

    /// Returns the [`Self::Sql`](InsertImpl::Sql) value generated by the macro derive.
    /// 
    /// Allows direct access to the SQL for further manipulation of the generated query.
    fn insert_sql(&self) -> Self::Sql<'_>;

    /// Returns the [`Insert::Query`](InsertImpl::Query) value built from the given SQL value.
    /// 
    /// Assumes [`Self::From`](InsertImpl::From) is based on the generated [`Self::Sql`](InsertImpl::Sql).
    fn insert_from<'q, 'a>(query: Self::From<'q, 'a>) -> Self::Query<'q, 'a>;
}
