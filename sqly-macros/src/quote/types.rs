use proc_macro2::TokenStream;
use syn::Result;

mod delete;
mod insert;
mod select;
mod update;

use crate::parse::*;
use crate::cache::*;

use std::fmt::Write;



impl TryFrom<QueryTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: QueryTable) -> Result<TokenStream> {
        cache::store().table(table)
    }
}

impl Cache for QueryTable {

    fn id(&self) -> Result<Id> {
        Id::try_from(&self.ident)
    }

    fn dep(&self) -> Result<Dep> {
        Ok(Dep::new())
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}



impl QueryTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let table = self.table()?;
        let types = self.types()?;

        let query = types.map(|t| self.query(t));
        let query = query.collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            #table
            #(#query)*
        })
    }

}



impl QueryTable {

    pub fn query(&self, r#type: Types) -> Result<TokenStream> {
        let derive = self.derive(r#type)?;
        let attr = self.attr(r#type)?;
        let vis = self.vis(r#type)?;
        let name = self.name(r#type)?;
        let fields = self.fields(r#type)?;

        let fields = fields.map(|f| f.stream(r#type));
        let fields = fields.collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            #[automatically_derived]
            #derive #attr
            #vis struct #name {
                #(#fields,)*
            }
        })
    }

}



impl QueryTable {

    pub fn table(&self) -> Result<TokenStream> {
        let ident = &self.ident;

        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::Table for #ident {}
        })
    }

}
