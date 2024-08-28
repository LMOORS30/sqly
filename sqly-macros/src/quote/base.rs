use proc_macro2::TokenStream;
use syn::Result;

use crate::parse::*;



impl QueryTable {

    pub fn attrs(&self, _: Types) -> Result<Vec<TokenStream>> {
        let ident = &self.ident;
        let table = &self.attr.table.data.data;
        let attrs = vectok![
            quote::quote! { table = #ident },
            quote::quote! { table_name = #table },
            self.attr.rename,
            self.attr.print,
            self.attr.debug,
        ];
        Ok(attrs)
    }

    pub fn attr(&self, r#type: Types) -> Result<TokenStream> {
        let attrs = self.attrs(r#type)?;
        let attr = match attrs.len() {
            0 => TokenStream::new(),
            _ => quote::quote! {
                #[sqly(#(#attrs),*)]
            },
        };
        Ok(attr)
    }

    pub fn derive(&self, r#type: Types) -> Result<TokenStream> {
        let derives = self.derives(r#type)?;
        let span = match r#type {
            Types::Delete => self.attr.delete.as_ref(),
            Types::Insert => self.attr.insert.as_ref(),
            Types::Select => self.attr.select.as_ref(),
            Types::Update => self.attr.update.as_ref(),
        }.map(|attr| attr.span).unwrap_or_else(|| {
            proc_macro2::Span::call_site()
        }); 
        let derive = match r#type {
            Types::Delete => quote::quote_spanned!{ span => ::sqly::Delete },
            Types::Insert => quote::quote_spanned!{ span => ::sqly::Insert },
            Types::Select => quote::quote_spanned!{ span => ::sqly::Select },
            Types::Update => quote::quote_spanned!{ span => ::sqly::Update },
        };
        Ok(quote::quote! { #[derive(#derive #(, #derives)*)] })
    }

}



impl QueryField {

    pub fn stream(&self, r#type: Types) -> Result<TokenStream> {
        let ty = &self.ty;
        let vis = &self.vis;
        let ident = &self.ident;

        let mut attrs = vectok![
            self.attr.column,
            self.attr.rename,
        ];

         match r#type {
             Types::Delete => {},
             Types::Insert => {},
             Types::Select => {},
             Types::Update => {
                 if self.keyed(r#type) {
                    let span = self.attr.key.as_ref().map(|key| {
                        key.data.iter().find(|val| {
                            r#type == val.data.into()
                        }).map_or(key.span, |val| val.span)
                    }).unwrap_or_else(proc_macro2::Span::call_site);
                    attrs.push(quote::quote_spanned! { span => key });
                 }
             },
        }

        let attr = match attrs.len() {
            0 => TokenStream::new(),
            _ => quote::quote! {
                #[sqly(#(#attrs),*)]
            },
        };

        Ok(quote::quote! { #attr #vis #ident: #ty })
    }

}



macro_rules! both {
($table:ty, $field:ty) => {
base!($table, $field);

impl $table {

    pub fn value(&self, field: &$field) -> Result<TokenStream> {
        let ident = &field.ident;
        let span = syn::spanned::Spanned::span(&field.ty);
        let value = quote::quote_spanned!(span => self.#ident);
        Ok(value)
    }

}

} }



macro_rules! base {
($table:ty, $field:ty) => {

impl $table {

    pub fn print(&self, query: &str, args: &[TokenStream])  -> Result<()> {
        if self.attr.print.is_some() {
            let mut tabs = String::new();
            for line in query.split('\n') {
                tabs.push_str("\n\t");
                tabs.push_str(line);
            }
            let mut vals = String::new();
            for arg in args {
                vals.push_str(",\n\t");
                vals.push_str(&arg.to_string())
            }
            println!("{}::query!(\n\tr#\"{}\n\t\"#{}\n)", self.ident, tabs, vals);
        }
        Ok(())
    }

    pub fn debug(&self, res: TokenStream) -> Result<TokenStream> {
        match self.attr.debug.as_ref() {
            Some(_) => match syn::parse2(res.clone()) {
                Ok(tree) => {
                    let rs = prettyplease::unparse(&tree);
                    println!("{}", rs);
                    Ok(res)
                },
                Err(err) => {
                    let rs = res.to_string();
                    println!("{}", rs);
                    Err(err)
                },
            },
            None => Ok(res),
        }
    }

}

} }



base!(QueryTable, QueryField);
both!(DeleteTable, DeleteField);
both!(InsertTable, InsertField);
both!(SelectTable, SelectField);
both!(UpdateTable, UpdateField);
