use proc_macro2::TokenStream;
use syn::Result;

mod delete;
mod insert;
mod select;
mod update;

use crate::parse::*;
use crate::cache::*;

use std::fmt::Write;



#[derive(Clone, Copy)]
pub enum Target {
    Function,
    Macro,
}

#[derive(Clone, Copy)]
pub enum Source {
    Column,
    Field,
}



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
        let mut dep = Dep::new();
        for column in self.coded()? {
            if let Code::Foreign(foreign) = column?.code {
                let id = Id::try_from(foreign.path)?;
                dep.end(Key::Table(id));
            }
        }
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}

impl QueryTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let ident = &self.ident;
        let db = db![];

        let mut local = Local::default();
        let local = self.colocate(&mut local)?;
        let construct = self.construct(local)?;

        let row = self.flat()?;
        let flat = construct.flat()?;
        let form = construct.form(Source::Column)?;
        let check = construct.check()?;
        let types = self.types()?;

        let types = types.map(|r#type| construct.build(r#type));
        let types = types.collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            #flat
            #[automatically_derived]
            impl ::sqly::Table for #ident {
                type DB = #db;
                type Flat = #row;
                fn from_row(row: <Self::DB as ::sqlx::Database>::Row) -> ::sqlx::Result<Self> {
                    #check
                    #form
                }
            }
            #(#types)*
        })
    }

}



impl<'c> Construct<'c> {

    pub fn build(&self, r#type: Types) -> Result<TokenStream> {
        let derive = self.table.derive(r#type)?;
        let attr = self.table.attr(r#type)?;
        let vis = self.table.vis(r#type)?;
        let name = self.table.name(r#type)?;

        let fields = self.fields.iter().filter(|column| {
            column.table.fielded(column.field, r#type)
        });

        let fields = fields.map(|column| {
            let ty = column.typed()?;
            let ident = column.named()?;
            let vis = &column.field.vis;
            let fttr = column.table.fttr(column.field, r#type)?;
            Ok(quote::quote! { #fttr #vis #ident: #ty })
        });

        let fields = fields.collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            #derive #attr
            #vis struct #name {
                #(#fields,)*
            }
        })
    }

}



impl<'c> Construct<'c> {

    pub fn flat(&self) -> Result<TokenStream> {
        let ident = &self.table.ident;
        let flat = self.table.flat()?;
        let db = db![];

        let vis = match &self.table.attr.flat_visibility {
            Some(vis) => &vis.data.data,
            None => &self.table.vis,
        };

        let derives = self.table.attr.flat_derive.iter().flat_map(|derive| {
            derive.data.iter().map(|data| &data.data)
        }).collect::<Vec<_>>();

        let derive = match derives.len() {
            0 => TokenStream::new(),
            _ => quote::quote! {
                #[derive(#(#derives),*)]
            }
        };

        let mut fields = Vec::new();
        let mut flatted = Vec::new();

        for flattened in self.flattened()? {
            let flattened = flattened?;
            let column = flattened.column;
            let optional = flattened.optional;

            if let Code::Query = column.code {
                let alias = column.alias()?;
                let ident = column.ident()?;
                flatted.push(quote::quote! {
                    #ident: row.try_get(#alias)?
                });

                let ty = column.typed()?;
                let vis = &column.field.vis;
                let ty = match (optional, column.table.optional(&column.field)?) {
                    (Some(option), None) => quote::quote! { #option<#ty> },
                    _ => quote::quote! { #ty },
                };
                fields.push(quote::quote! {
                    #vis #ident: #ty
                });
            }
        }

        let form = self.form(Source::Field)?;

        Ok(quote::quote! {
            #derive #vis struct #flat {
                #(#fields,)*
            }
            #[automatically_derived]
            impl<'r> ::sqlx::FromRow<'r, <#db as ::sqlx::Database>::Row> for #flat {
                fn from_row(row: &'r <#db as ::sqlx::Database>::Row) -> ::sqlx::Result<Self> {
                    use ::sqlx::Row as _;
                    Ok(#flat { #(#flatted,)* })
                }
            }
            #[automatically_derived]
            impl From<#flat> for #ident {
                fn from(row: #flat) -> Self {
                    #form
                }
            }
        })
    }

}



struct Former<'c> {
    option: &'c syn::Path,
    ident: syn::Ident,
    alias: &'c str,
}

impl<'c> Construct<'c> {

    pub fn form(&self, source: Source) -> Result<TokenStream> {
        let formed = self.formed(None, source)?;

        match source {
            Source::Field => {
                Ok(quote::quote! {
                    #formed
                })
            },
            Source::Column => {
                Ok(quote::quote! {
                    use ::sqlx::Row as _;
                    Ok(#formed)
                })
            }
        }
    }

    fn formed(&self, former: Option<&Former<'c>>, source: Source) -> Result<TokenStream> {
        let fields = self.fields.iter().map(|column| {
            let value = match &column.code {
                Code::Skip => {
                    quote::quote! { ::core::default::Default::default() }
                },
                Code::Query => match source {
                    Source::Field => {
                        let unwrap = match former {
                            Some(former) => match column.table.optional(column.field)? {
                                None => Some(former.option),
                                Some(_) => None,
                            },
                            None => None,
                        };
                        let ident = column.ident()?;
                        match unwrap {
                            None => quote::quote! { row.#ident },
                            Some(option) => quote::quote! {
                                match row.#ident {
                                    #option::Some(val) => val,
                                    #option::None => break #option::None,
                                }
                            },
                        }
                    },
                    Source::Column => {
                        let param = match former {
                            None => None,
                            Some(former) => {
                                let ident = column.ident()?;
                                match ident.eq(&former.ident) {
                                    true => Some(ident),
                                    false => None,
                                }
                            }
                        };
                        match param {
                            Some(param) => quote::quote! { #param },
                            None => {
                                let alias = column.alias()?;
                                quote::quote! { row.try_get(#alias)? }
                            }
                        }
                    }
                },
                Code::Foreign(construct) => {
                    let optional = match construct.optional()? {
                        None => None,
                        Some(option) => {
                            let field = match construct.constitute()? {
                                Some(field) => field,
                                None => {
                                    let ident = &construct.table.ident;
                                    let span = syn::spanned::Spanned::span(&column.field.ty);
                                    let msg = format!("ambiguous left join on {ident}: \
                                        all fetched fields are optional");
                                    return Err(syn::Error::new(span, msg));
                                }
                            };
                            let alias = field.alias()?;
                            let ident = field.ident()?;
                            Some(Former { option, ident, alias })
                        }
                    };
                    let former = optional.as_ref().or(former);
                    let formed = construct.formed(former, source)?;
                    match optional {
                        None => formed,
                        Some(former) => match source {
                            Source::Field => {
                                let option = former.option;
                                quote::quote! { loop { break #option::Some(#formed); } }
                            },
                            Source::Column => {
                                let alias = &former.alias;
                                let ident = &former.ident;
                                let option = &former.option;
                                quote::quote! {
                                    match row.try_get::<#option<_>, _>(#alias)? {
                                        #option::Some(#ident) => #option::Some(#formed),
                                        #option::None => #option::None,
                                    }
                                }
                            }
                        }
                    }
                }
            };
            let ident = &column.field.ident;
            Ok(quote::quote! { #ident: #value })
        }).collect::<Result<Vec<_>>>()?;

        let path = match &self.foreign {
            Some(foreign) => {
                let path = foreign.path;
                quote::quote! { #path }
            },
            None => {
                let ident = &self.table.ident;
                quote::quote! { #ident }
            }
        };

        Ok(quote::quote! {
            #path { #(#fields,)* }
        })
    }

}
