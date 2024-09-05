#[allow(unused_imports)]
use super::*;



/// A type which can select rows from a table.
/// 
/// This trait is not meant to be manually implemented,
/// see [`#[derive(Select)]`](derive@Select)
/// or [`#[sqly(select)]`](derive@Table#select)
/// instead.
pub trait Select {
    /// The type representing the table which this query will operate on.
    /// 
    /// When generated with [`#[derive(Select)]`](derive@Select) this type is set by the [`#[sqly(table = Ident)]`](derive@Table#table) attribute.
    /// 
    /// When generated with [`#[sqly(select)]`](derive@Table#select) this type is set to the struct for which [`#[derive(Table)]`](derive@Table) was called.
    /// 
    /// As this type implements the [`Table`](Table) trait the [`Table::select`](`Table::select`) alias is made available, this type serves no other purpose.
    type Table: Table;

    /// The query type for the operation to be executed.
    /// 
    /// This will be equal to [`sqlx::query::Map`], with the `DB` type defined by the features enabled for this crate (see [Features](crate#features)).
    type Query<'q>
        where Self: 'q;

    /// Returns a query which selects rows from the table according to the definition of this type.
    /// 
    /// This function is not meant to be manualy implemented,
    /// see [`#[derive(Select)]`](derive@Select)
    /// or [`#[sqly(select)]`](derive@Table#select)
    /// instead.
    fn select(&self) -> Self::Query<'_>;
}
