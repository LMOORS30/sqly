use proc_macro2::TokenStream;
use syn::Result;

use crate::cache::*;
use crate::parse::*;
use crate::quote::*;



impl QueryTable {

    pub fn attrs(&self, _: Types) -> Result<Vec<TokenStream>> {
        let ident = &self.ident;
        let attrs = vectok![
            quote::quote! { table = #ident },
            self.attr.rename,
            self.attr.unchecked,
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
            }
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

    pub fn fttrs(&self, field: &QueryField, r#type: Types) -> Result<Vec<TokenStream>> {
        let mut fttrs = vectok![
            field.attr.column,
            field.attr.rename,
            field.attr.infer,
            field.attr.value,
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
                    let key = quote::quote_spanned! { span => key };
                    fttrs.insert(0, key);
                }
            }
        }
        Ok(fttrs)
    }

    pub fn fttr(&self, field: &QueryField, r#type: Types) -> Result<TokenStream> {
        let fttrs = self.fttrs(field, r#type)?;
        let fttr = match fttrs.len() {
            0 => TokenStream::new(),
            _ => quote::quote! {
                #[sqly(#(#fttrs),*)]
            }
        };
        Ok(fttr)
    }

}



impl<'c> Construct<'c> {

    #[cfg(feature = "unchecked")]
    pub fn check(&self) -> Result<TokenStream> {
        Ok(TokenStream::new())
    }

    #[cfg(not(feature = "unchecked"))]
    pub fn check(&self) -> Result<TokenStream> {
        if self.table.attr.unchecked.is_some() {
            return Ok(TokenStream::new());
        }

        use std::fmt::Write;
        let name = &self.table.ident;
        let table = &self.table.attr.table.data.data;
        let mut query = String::new();
        let mut fields = Vec::new();
        let unique = self.unique()?;

        write!(&mut query,
            "SELECT\n"
        ).unwrap();

        let mut i = 1;
        for column in self.fields.iter() {
            if let Code::Query = column.code {
                let alias = column.alias()?;
                let modifier = column.modifier()?;
                let alias = format!("{alias}{modifier}");
                let list = column.table.selects(column.field)?;
                if !list.is_empty() {
                    query.push_str("\t");
                    let params = self.selects(&alias)?;
                    let select = list.into_iter().map(|select| {
                        params.replace(&select.data, select.span)
                    }).collect::<Result<String>>()?;
                    query.push_str(&select);
                    query.push_str(",\n");
                }
                else {
                    let column = column.column()?;
                    write!(&mut query,
                        "\t\"{column}\" AS \"{alias}\",\n"
                    ).unwrap();
                }
                let ty = column.typed()?;
                let ident = column.ident()?;
                fields.push(quote::quote! {
                    #ident: #ty
                });
                i += 1;
            }
        }
        if i <= 1 { return Ok(TokenStream::new()); }
        let trunc = if i > 1 { 2 } else { 1 };
        query.truncate(query.len() - trunc);

        write!(&mut query,
            "\nFROM \"{table}\" AS \"{unique}\""
        ).unwrap();

        Ok(quote::quote! {
            #[allow(unused)]
            fn __() { 
                struct #name { #(#fields,)* }
                ::sqlx::query_as!(#name, #query);
            }
        })
    }

}



macro_rules! base {
($table:ty, $field:ty) => {
both!($table, $field);

impl $table {

    #[cfg(feature = "unchecked")]
    pub fn checked<F>(&self, _: &[&$field], _: F) -> Result<TokenStream>
    where F: FnOnce(&[TokenStream]) -> Result<TokenStream> {
        Ok(TokenStream::new())
    }

    #[cfg(not(feature = "unchecked"))]
    pub fn checked<F>(&self, args: &[&$field], cb: F) -> Result<TokenStream>
    where F: FnOnce(&[TokenStream]) -> Result<TokenStream> {
        if self.attr.unchecked.is_some() {
            return Ok(TokenStream::new());
        }
        let obj = &self.ident;
        let args = args.iter().map(|field| {
            self.value(field, Target::Macro)
        }).collect::<Result<Vec<_>>>()?;
        let check = cb(&args)?;
        Ok(quote::quote! {
            #[allow(unused)]
            fn __(obj: &#obj) {
                #check
            }
        })
    }

    pub fn check(&self, query: &str, args: &[&$field]) -> Result<TokenStream> {
        self.checked(args, |args| Ok(quote::quote! {
            ::sqlx::query!(#query #(, #args)*);
        }))
    }

}

} }



macro_rules! both {
($table:ty, $field:ty) => {

impl $table {

    pub fn value(&self, field: &$field, target: Target) -> Result<proc_macro2::TokenStream> {
        let span = syn::spanned::Spanned::span(&field.ty);
        let value = match &field.attr.value {
            Some(expr) => {
                let expr = &expr.data.data;
                let unfer = unfer(expr);
                let unfer = unfer.as_ref().unwrap_or(expr);
                match (target, &field.attr.infer) {
                    (Target::Macro, None) => quote::quote_spanned!(span => #expr),
                    (Target::Macro, Some(_)) => quote::quote_spanned!(span => (#unfer) as _),
                    (Target::Function, _) => quote::quote_spanned!(span => #unfer),
                }
            },
            None => {
                let ident = &field.ident;
                match &field.attr.infer.as_ref().map(|_| target) {
                    Some(Target::Macro) => quote::quote_spanned!(span => obj.#ident as _),
                    Some(Target::Function) | None => quote::quote_spanned!(span => obj.#ident),
                }
            }
        };
        Ok(value)
    }

}

impl $table {

    pub fn print(&self, query: &str, args: &[&$field])  -> Result<()> {
        if self.attr.print.is_some() {
            let mut tabs = String::new();
            for line in query.split('\n') {
                tabs.push_str("\n\t");
                tabs.push_str(line);
            }
            let mut vals = String::new();
            for arg in args {
                vals.push_str(",\n\t");
                let val = self.value(arg, Target::Macro)?;
                vals.push_str(&val.to_string())
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
