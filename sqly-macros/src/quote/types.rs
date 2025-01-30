use proc_macro2::TokenStream;

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

#[derive(Clone, Copy)]
pub enum Scope {
    Global,
    Local,
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
            #check
            #[automatically_derived]
            impl ::sqly::Table for #ident {
                type DB = #db;
                type Flat = #row;
                fn from_row(row: <Self::DB as ::sqlx::Database>::Row) -> ::sqlx::Result<Self> {
                    #form
                }
            }
            #(#types)*
        })
    }

}



impl Construct<'_> {

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



impl Construct<'_> {

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

        for flattened in self.flatten()? {
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
                let ty = match (optional, column.optional()?) {
                    (Some(optional), None) => {
                        let option = optional.option()?;
                        quote::quote! { #option<#ty> }
                    }
                    _ => quote::quote! { #ty },
                };
                fields.push(quote::quote! {
                    #vis #ident: #ty
                });
            }
        }

        let form = self.form(Source::Field)?;

        Ok(quote::quote! {
            #[allow(non_snake_case)]
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
    target: &'c Constructed<'c>,
    source: &'c Constructed<'c>,
    option: Optional<'c>,
    ident: syn::Ident,
}

impl Construct<'_> {

    pub fn form(&self, source: Source) -> Result<TokenStream> {
        let formed = self.formed(None, source)?;

        match source {
            Source::Field => {
                Ok(quote::quote! {
                    #formed
                })
            }
            Source::Column => {
                Ok(quote::quote! {
                    use ::sqlx::Row as _;
                    Ok(#formed)
                })
            }
        }
    }

    fn formed(&self, former: Option<&Former<'_>>, source: Source) -> Result<TokenStream> {
        let fields = self.fields.iter().map(|column| {
            let value = match &column.code {
                Code::Skip => {
                    let default = column.field.attr.default.as_ref();
                    match default.and_then(|data| data.data.as_ref()) {
                        None => quote::quote! { ::core::default::Default::default() },
                        Some(default) => quote::quote! { #default() },
                    }
                }
                Code::Query => match source {
                    Source::Field => match (former, column.optional()?) {
                        (Some(former), None) => {
                            let optional = &former.option;
                            let from = column.from()?;
                            let ident = column.ident()?;
                            let option = optional.option()?;
                            let default = optional.default()?;
                            let none = former.source.none()?;
                            quote::quote! {
                                match row.#ident {
                                    #option::Some(val) => #from(val),
                                    #option::None => break #none(#default),
                                }
                            }
                        }
                        (_, Some(Optional::Default(path))) => {
                            let optional = Optional::Default(path);
                            let from = column.from()?;
                            let ident = column.ident()?;
                            let option = optional.option()?;
                            let default = optional.default()?;
                            quote::quote! {
                                match row.#ident {
                                    #option::Some(val) => #from(val),
                                    #option::None => #default,
                                }
                            }
                        }
                        _ => {
                            let from = column.from()?;
                            let ident = column.ident()?;
                            quote::quote! { #from(row.#ident) }
                        }
                    }
                    Source::Column => {
                        let ident = column.ident()?;
                        let param = former.and_then(|former| {
                            match ident.eq(&former.ident) {
                                true => Some(ident),
                                false => None,
                            }
                        });
                        match param {
                            Some(param) => {
                                let from = column.from()?;
                                quote::quote! { #from(#param) }
                            }
                            None => match column.optional()? {
                                Some(Optional::Default(path)) => {
                                    let optional = Optional::Default(path);
                                    let ty = column.typed()?;
                                    let from = column.from()?;
                                    let alias = column.alias()?;
                                    let option = optional.option()?;
                                    let default = optional.default()?;
                                    quote::quote! {
                                        match row.try_get::<#ty, _>(#alias)? {
                                            #option::Some(val) => #from(val),
                                            #option::None => #default,
                                        }
                                    }
                                }
                                _ => {
                                    let ty = column.typed()?;
                                    let from = column.from()?;
                                    let alias = column.alias()?;
                                    quote::quote! { #from(row.try_get::<#ty, _>(#alias)?) }
                                }
                            }
                        }
                    }
                }
                Code::Foreign(construct) => {
                    let optional = match construct.optional()? {
                        None => None,
                        Some(option) => {
                            let field = match construct.constitute()? {
                                Some(field) => field,
                                None => {
                                    let ident = &construct.table.ident;
                                    let span = column.table.ty(column.field)?.span();
                                    let msg = format!("ambiguous left join on {ident}: \
                                        all fetched fields are optional");
                                    return Err(syn::Error::new(span, msg));
                                }
                            };
                            let target = &field;
                            let source = &column;
                            let ident = field.ident()?;
                            Some(Former { option, source, target, ident })
                        }
                    };
                    let former = optional.as_ref().or(former);
                    let formed = construct.formed(former, source)?;
                    match optional {
                        None => {
                            let from = column.from()?;
                            quote::quote! { #from(#formed) }
                        }
                        Some(former) => match source {
                            Source::Field => {
                                let from = column.from()?;
                                let some = former.option.some()?;
                                quote::quote! { loop { break #from(#some(#formed)); } }
                            }
                            Source::Column => {
                                let ident = &former.ident;
                                let optional = &former.option;
                                let ty = former.target.typed()?;
                                let alias = former.target.alias()?;
                                let default = optional.default()?;
                                let option = optional.option()?;
                                let some = optional.some()?;
                                let none = column.none()?;
                                let from = column.from()?;
                                quote::quote! {
                                    match row.try_get::<#option<#ty>, _>(#alias)? {
                                        #option::Some(#ident) => #from(#some(#formed)),
                                        #option::None => #none(#default),
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
            }
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
