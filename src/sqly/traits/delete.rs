#[allow(unused_imports)]
use super::*;



/// A type which can delete rows from a table.
/// 
/// This trait is not meant to be manually implemented,
/// see [`#[derive(Delete)]`](derive@Delete)
/// or [`#[sqly(delete)]`](docs::attr#delete)
/// instead.
pub trait Delete {
    /// The type representing the table which this query will operate on.
    /// 
    /// When generated with [`#[derive(Delete)]`](derive@Delete) this type is set by the [`#[sqly(table)]`](docs::attr#table) attribute.
    /// 
    /// When generated with [`#[sqly(delete)]`](docs::attr#delete) this type is set to the struct for which [`#[derive(Table)]`](derive@Table) was called.
    /// 
    /// When this type implements the [`Table`](Table) trait the [`Table::delete`](`Table::delete`) alias is made available, this type serves no other purpose.
    type Table;

    /// The query type for the operation to be executed.
    /// 
    /// This will be equal to [`sqlx::query::Query`], with the `DB` type defined by the features enabled for this crate (see [Features](https://github.com/LMOORS30/sqly#features)).
    type Query<'q>
        where Self: 'q;

    /// Returns a query which deletes rows from the table according to the definition of this type.
    /// 
    /// This function is not meant to be manualy implemented,
    /// see [`#[derive(Delete)]`](derive@Delete)
    /// or [`#[sqly(delete)]`](docs::attr#delete)
    /// instead.
    fn delete(&self) -> Self::Query<'_>;
}
