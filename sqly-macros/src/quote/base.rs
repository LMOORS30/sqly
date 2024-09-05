use proc_macro2::TokenStream;
use syn::Result;

use crate::parse::*;



#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Query,
    Check,
}



impl QueryTable {

    pub fn attrs(&self, _: Types) -> Result<Vec<TokenStream>> {
        let ident = &self.ident;
        let attrs = vectok![
            quote::quote! { table = #ident },
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



impl QueryTable {

    pub fn fttr(&self, field: &QueryField, r#type: Types) -> Result<TokenStream> {
        let mut attrs = vectok![
            field.attr.column,
            field.attr.rename,
        ];

         match r#type {
            Types::Delete => {},
            Types::Insert => {},
            Types::Select => {},
            Types::Update => {
                if self.keyed(field, r#type) {
                    let span = field.attr.key.as_ref().map(|key| {
                        key.data.iter().find(|val| {
                            r#type == val.data.into()
                        }).map_or(key.span, |val| val.span)
                    }).unwrap_or_else(proc_macro2::Span::call_site);
                    attrs.push(quote::quote_spanned! { span => key });
                }
            },
        }

        match attrs.len() {
            0 => Ok(TokenStream::new()),
            _ => Ok(quote::quote! {
                #[sqly(#(#attrs),*)]
            })
        }
    }

    pub fn field(&self, field: &QueryField, target: Target) -> Result<TokenStream> {
        let ty = &field.ty;
        let vis = &field.vis;
        match target {
            Target::Query => {
                let ident = &field.ident;
                Ok(quote::quote! { #vis #ident: #ty })
            },
            Target::Check => {
                let column = self.column(field, Target::Query)?;
                let ident = quote::format_ident!("{column}");
                Ok(quote::quote! { #vis #ident: #ty })
            }
        }
    }

}



macro_rules! base {
($table:ty, $field:ty) => {
both!($table, $field);

impl $table {

}

} }



macro_rules! both {
($table:ty, $field:ty) => {

impl $table {

    pub fn column(&self, field: &$field, target: Target) -> Result<String> {
        let name = match &field.attr.column {
            Some(column) => column.data.data.clone(),
            None => field.ident.to_string(),
        };

        const SEP: &'static [char] = &['!', '?', ':'];
        let (name, info) = match name.find(SEP) {
            Some(i) => name.split_at(i),
            None => (name.as_str(), ""),
        };

        let all = &self.attr.rename;
        let rename = &field.attr.rename;
        let name = match rename.as_ref().or(all.as_ref()) {
            Some(re) => re.data.data.rename(name),
            None => name.to_owned(),
        };

        match target {
            Target::Query => Ok(name),
            Target::Check => {
                let mut name = name;
                name.push_str(info);
                Ok(name)
            }
        }
    }

    pub fn value(&self, field: &$field, target: Target) -> Result<TokenStream> {
        let span = syn::spanned::Spanned::span(&field.ty);
        let ident = &field.ident;
        let value = match target {
            Target::Query => quote::quote_spanned!(span => self.#ident),
            Target::Check => quote::quote_spanned!(span => item.#ident),
        };
        Ok(value)
    }

}



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



both!(QueryTable, QueryField);
base!(DeleteTable, DeleteField);
base!(InsertTable, InsertField);
base!(SelectTable, SelectField);
base!(UpdateTable, UpdateField);
