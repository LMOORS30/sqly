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
/// ([`Delete`], [`Insert`], [`Select`], [`Update`]) where `<Table = Self>`.
pub trait Table {

    /// Returns a query which deletes rows from the table according to the definition of the given type.
    /// 
    /// This function is not meant to be implemented
    /// and instead delegates to [`Delete::delete`].
    fn delete<R>(row: &R) -> R::Query<'_>
    where R: Delete<Table = Self> {
        row.delete()
    }

    /// Returns a query which inserts rows into the table according to the definition of the given type.
    /// 
    /// This function is not meant to be implemented
    /// and instead delegates to [`Insert::insert`].
    fn insert<R>(row: &R) -> R::Query<'_>
    where R: Insert<Table = Self> {
        row.insert()
    }

    /// Returns a query which selects rows from the table according to the definition of the given type.
    /// 
    /// This function is not meant to be implemented
    /// and instead delegates to [`Select::select`].
    fn select<R>(row: &R) -> R::Query<'_>
    where R: Select<Table = Self> {
        row.select()
    }

    /// Returns a query which updates rows in the table according to the definition of the given type.
    /// 
    /// This function is not meant to be implemented
    /// and instead delegates to [`Update::update`].
    fn update<R>(row: &R) -> R::Query<'_>
    where R: Update<Table = Self> {
        row.update()
    }

}
