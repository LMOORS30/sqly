use proc_macro2::TokenStream;
use syn::DeriveInput;

use crate::parse::*;

pub fn table(input: DeriveInput) -> Result<TokenStream> {
    let table = QueryTable::try_from(input)?;
    TokenStream::try_from(table.init()?)
}

pub fn delete(input: DeriveInput) -> Result<TokenStream> {
    let table = DeleteTable::try_from(input)?;
    TokenStream::try_from(table.init()?)
}

pub fn insert(input: DeriveInput) -> Result<TokenStream> {
    let table = InsertTable::try_from(input)?;
    TokenStream::try_from(table.init()?)
}

pub fn select(input: DeriveInput) -> Result<TokenStream> {
    let table = SelectTable::try_from(input)?;
    TokenStream::try_from(table.init()?)
}

pub fn update(input: DeriveInput) -> Result<TokenStream> {
    let table = UpdateTable::try_from(input)?;
    TokenStream::try_from(table.init()?)
}
