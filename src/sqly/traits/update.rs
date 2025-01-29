#[allow(unused_imports)]
use super::*;



/// A type which can update rows in a table.
/// 
/// This trait is not meant to be manually implemented,
/// see [`#[derive(Update)]`](derive@Update)
/// or [`#[sqly(update)]`](docs::attr#update)
/// instead.
pub trait Update {
    /// The type representing the table which this query will operate on.
    /// 
    /// When generated with [`#[derive(Update)]`](derive@Update) this type is set by the [`#[sqly(table = Ident)]`](docs::attr#table) attribute.
    /// 
    /// When generated with [`#[sqly(update)]`](docs::attr#update) this type is set to the struct for which [`#[derive(Table)]`](derive@Table) was called.
    /// 
    /// As this type implements the [`Table`](Table) trait the [`Table::update`](`Table::update`) alias is made available, this type serves no other purpose.
    type Table: Table;

    /// The query type for the operation to be executed.
    /// 
    /// This will be equal to [`sqlx::query::Query`], with the `DB` type defined by the features enabled for this crate (see [Features](https://github.com/LMOORS30/sqly#features)).
    type Query<'q>
        where Self: 'q;

    /// Returns a query which updates rows in the table according to the definition of this type.
    /// 
    /// This function is not meant to be manualy implemented,
    /// see [`#[derive(Update)]`](derive@Update)
    /// or [`#[sqly(update)]`](docs::attr#update)
    /// instead.
    fn update(&self) -> Self::Query<'_>;
}
