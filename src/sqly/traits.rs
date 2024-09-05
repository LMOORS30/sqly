#[allow(unused_imports)]
use super::*;

mod delete;
mod insert;
mod select;
mod update;

pub use delete::*;
pub use insert::*;
pub use select::*;
pub use update::*;



/// A type which represents a database table.
/// 
/// This trait is not meant to be manually implemented,
/// see [`#[derive(Table)]`](derive@Table) instead.
/// 
/// This trait serves as an alias to the implementation of other traits
/// ([`Delete`](Delete), [`Insert`](Insert), [`Select`](Select), [`Update`](Update)) where `<Table = Self>`.
pub trait Table {
    /// The sqlx database type for which queries are built.
    /// 
    /// This will be equal to the type defined by the features enabled for this crate (see [Features](crate#features)).
    type DB: sqlx::Database;

    /// Returns a table record built from a row returned by the database.
    /// 
    /// This function is not meant to be manualy implemented,
    /// see [`#[derive(Table)]`](derive@Table) instead.
    fn from_row(row: <Self::DB as sqlx::Database>::Row) -> sqlx::Result<Self>
    where Self: Sized;

    /// Returns a query which deletes rows from the table according to the definition of the given type.
    /// 
    /// This function is not meant to be implemented
    /// and instead delegates to [`Delete::delete`](Delete::delete).
    fn delete<R>(row: &R) -> R::Query<'_>
    where R: Delete<Table = Self> {
        row.delete()
    }

    /// Returns a query which inserts rows into the table according to the definition of the given type.
    /// 
    /// This function is not meant to be implemented
    /// and instead delegates to [`Insert::insert`](Insert::insert).
    fn insert<R>(row: &R) -> R::Query<'_>
    where R: Insert<Table = Self> {
        row.insert()
    }

    /// Returns a query which selects rows from the table according to the definition of the given type.
    /// 
    /// This function is not meant to be implemented
    /// and instead delegates to [`Select::select`](Select::select).
    fn select<R>(row: &R) -> R::Query<'_>
    where R: Select<Table = Self> {
        row.select()
    }

    /// Returns a query which updates rows in the table according to the definition of the given type.
    /// 
    /// This function is not meant to be implemented
    /// and instead delegates to [`Update::update`](Update::update).
    fn update<R>(row: &R) -> R::Query<'_>
    where R: Update<Table = Self> {
        row.update()
    }

}
