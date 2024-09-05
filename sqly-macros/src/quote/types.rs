use proc_macro2::TokenStream;
use syn::Result;

mod delete;
mod insert;
mod select;
mod update;

use super::base::*;
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
        let types = self.types()?;
        let query = self.query()?;
        let ident = &self.ident;
        let db = db![];

        let typed = types.map(|t| self.typed(t));
        let typed = typed.collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::Table for #ident {
                type DB = #db;
                fn from_row(row: <Self::DB as ::sqlx::Database>::Row) -> ::sqlx::Result<Self> {
                    #query
                }
            }
            #(#typed)*
        })
    }

}



impl QueryTable {

    pub fn typed(&self, r#type: Types) -> Result<TokenStream> {
        let derive = self.derive(r#type)?;
        let attr = self.attr(r#type)?;
        let vis = self.vis(r#type)?;
        let name = self.name(r#type)?;
        let fields = self.fields(r#type)?;

        let typed = fields.map(|field| {
            let fttr = self.fttr(field, r#type)?;
            let field = self.field(field, Target::Query)?;
            Ok(quote::quote!{ #fttr #field })
        });

        let typed = typed.collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            #[automatically_derived]
            #derive #attr
            #vis struct #name {
                #(#typed,)*
            }
        })
    }

}



impl QueryTable {

    pub fn query(&self) -> Result<TokenStream> {
        let fields = self.fields.iter();

        let fields = fields.map(|field| {
            let ident = &field.ident;

            match self.skipped(field, Skips::Query) {
                true => {
                    Ok(quote::quote! {
                        #ident: ::core::default::Default::default()
                    })
                }
                false => {
                    let column = self.column(field, Target::Query)?;
                    Ok(quote::quote! {
                        #ident: row.try_get(#column)?
                    })
                }
            }
        });

        let fields = fields.collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            use ::sqlx::Row;
            Ok(Self { #(#fields,)* })
        })
    }

}
