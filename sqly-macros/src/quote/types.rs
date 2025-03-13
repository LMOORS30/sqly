use super::*;

mod delete;
mod insert;
mod select;
mod update;



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
        let db = db![];
        let ident = &self.ident;
        let krate = self.krate()?;

        let mut local = Local::default();
        let local = self.colocate(&mut local)?;
        let construct = self.construct(local)?;

        let check = construct.check()?;
        let flat = match &self.attr.flat {
            Some(_) => Some(construct.flat()?),
            None => None,
        };
        let form = match spany!(self.attr.from_row, self.attr.select) {
            Some(_) => Some(construct.form(Source::Column)?),
            None => None,
        };

        let from_row = form.map(|form| quote::quote! {
            #[automatically_derived]
            impl<'r> #krate::sqlx::FromRow<'r, <#krate #db as #krate::sqlx::Database>::Row> for #ident {
                fn from_row(row: &'r <#krate #db as #krate::sqlx::Database>::Row) -> #krate::sqlx::Result<Self> {
                    #![allow(non_snake_case)]
                    use #krate::sqlx::Row as _;
                    #krate::sqlx::Result::Ok(#form)
                }
            }
        });

        let types = self.types()?;
        let types = types.map(|r#type| construct.build(r#type));
        let types = types.collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            #flat
            #check
            #from_row
            #[automatically_derived]
            impl #krate::Table for #ident {}
            #(#types)*
        })
    }

}



impl Construct<'_> {

    pub fn build(&self, r#type: Types) -> Result<TokenStream> {
        let derive = self.table.derive(r#type.into())?;
        let name = self.table.name(r#type.into())?;
        let vis = self.table.vis(r#type.into())?;
        let attr = self.table.attr(r#type)?;

        let fields = self.fields.iter().filter(|column| {
            column.table.fielded(column.field, r#type)
        }).map(|column| {
            let ty = column.retyped(r#type)?;
            let ident = column.renamed()?;
            let vis = &column.field.vis;
            let mut fttr = column.table.fttr(column.field, r#type)?;
            if let Some(span) = column.table.attr.serde_double_option.spany() {
                if optype(&syn::parse2(ty.clone())?).and_then(|(_, ty)| optype(ty)).is_some() {
                    let with = format!("{}::double_option", self.table.krate()?);
                    fttr = quote::quote_spanned!(span =>
                        #[serde(
                            default,
                            with = #with,
                            skip_serializing_if = "::core::option::Option::is_none"
                        )]
                        #fttr
                    );
                }
            }
            Ok(quote::quote! { #fttr #vis #ident: #ty })
        }).collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            #derive #attr
            #vis struct #name {
                #(#fields,)*
            }
        })
    }

}



impl Construct<'_> {

    pub fn flats(&self) -> Result<Vec<TokenStream>> {
        let mut flats = Vec::new();
        for flattened in self.flatten()? {
            let flattened = flattened?;
            let column = flattened.column;
            let nullable = flattened.nullable;
            if let Code::Query = column.code {
                let ty = column.typed()?;
                let ident = column.ident()?;
                let vis = &column.field.vis;
                let ty = match (nullable, column.nullable()?) {
                    (Some(nullable), None) => {
                        let option = nullable.option()?;
                        quote::quote! { #option<#ty> }
                    }
                    _ => quote::quote! { #ty },
                };
                flats.push(quote::quote! {
                    #vis #ident: #ty
                });
            }
        }
        Ok(flats)
    }

    pub fn flat(&self) -> Result<TokenStream> {
        let db = db![];
        let ident = &self.table.ident;
        let krate = self.table.krate()?;

        let derive = self.table.derive(Structs::Flat)?;
        let flat = self.table.name(Structs::Flat)?;
        let vis = self.table.vis(Structs::Flat)?;
        let flats = self.flats()?;

        let sets = match &self.table.attr.flat_row {
            None => None,
            Some(_) => {
                let mut sets = Vec::new();
                for flattened in self.flatten()? {
                    let flattened = flattened?;
                    let column = flattened.column;
                    if let Code::Query = column.code {
                        let alias = column.alias()?;
                        let ident = column.ident()?;
                        sets.push(quote::quote! {
                            #ident: row.try_get(#alias)?
                        });
                    }
                }
                Some(sets)
            }
        };
        let flat_row = sets.map(|sets| quote::quote! {
            #[automatically_derived]
            impl<'r> #krate::sqlx::FromRow<'r, <#krate #db as #krate::sqlx::Database>::Row> for #flat {
                fn from_row(row: &'r <#krate #db as #krate::sqlx::Database>::Row) -> #krate::sqlx::Result<Self> {
                    use #krate::sqlx::Row as _;
                    #krate::sqlx::Result::Ok(#flat { #(#sets,)* })
                }
            }
        });

        let form = match &self.table.attr.from_flat {
            Some(_) => Some(self.form(Source::Field)?),
            None => None,
        };
        let from_flat = form.map(|form| quote::quote! {
            #[automatically_derived]
            impl ::core::convert::From<#flat> for #ident {
                fn from(row: #flat) -> Self {
                    #form
                }
            }
        });

        Ok(quote::quote! {
            #[allow(non_snake_case)]
            #derive #vis struct #flat {
                #(#flats,)*
            }
            #flat_row
            #from_flat
            #[automatically_derived]
            impl #krate::Flat for #ident {
                type Flat = #flat;
            }
        })
    }

}



struct Former<'c> {
    target: &'c Constructed<'c>,
    source: &'c Constructed<'c>,
    option: Nullable<'c>,
    ident: syn::Ident,
}

impl Construct<'_> {

    pub fn form(&self, source: Source) -> Result<TokenStream> {
        self.formed(None, source)
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
                    Source::Field => match (former, column.nullable()?) {
                        (Some(former), None) => {
                            let nullable = &former.option;
                            let from = column.from()?;
                            let ident = column.ident()?;
                            let option = nullable.option()?;
                            let default = nullable.default()?;
                            let none = former.source.none()?;
                            quote::quote! {
                                match row.#ident {
                                    #option::Some(val) => #from(val),
                                    #option::None => break #none(#default),
                                }
                            }
                        }
                        (_, Some(Nullable::Default(path))) => {
                            let nullable = Nullable::Default(path);
                            let from = column.from()?;
                            let ident = column.ident()?;
                            let option = nullable.option()?;
                            let default = nullable.default()?;
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
                            None => match column.nullable()? {
                                Some(Nullable::Default(path)) => {
                                    let nullable = Nullable::Default(path);
                                    let ty = column.typed()?;
                                    let from = column.from()?;
                                    let alias = column.alias()?;
                                    let option = nullable.option()?;
                                    let default = nullable.default()?;
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
                    let nullable = match construct.nullable()? {
                        None => None,
                        Some(option) => {
                            let field = match construct.constitute()? {
                                Some(field) => field,
                                None => {
                                    let ident = &construct.table.ident;
                                    let span = column.table.ty(column.field)?.span();
                                    let msg = format!("ambiguous left join on {ident}: \
                                        all fetched fields are nullable");
                                    return Err(syn::Error::new(span, msg));
                                }
                            };
                            let target = &field;
                            let source = &column;
                            let ident = field.ident()?;
                            Some(Former { option, source, target, ident })
                        }
                    };
                    let former = nullable.as_ref().or(former);
                    let formed = construct.formed(former, source)?;
                    match nullable {
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
                                let nullable = &former.option;
                                let ty = former.target.typed()?;
                                let alias = former.target.alias()?;
                                let default = nullable.default()?;
                                let option = nullable.option()?;
                                let some = nullable.some()?;
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
