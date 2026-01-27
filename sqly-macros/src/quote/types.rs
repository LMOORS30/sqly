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

    fn dep(&self) -> Result<Dep<'_>> {
        let mut dep = Dep::new();
        for column in self.coded()? {
            if let Code::Foreign(foreign) = column?.code {
                dep.end(Key::Table(foreign.path));
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
        let row = row![];
        let ident = &self.ident;
        let krate = self.krate()?;

        let mut local = Local::default();
        let local = self.colocate(&mut local)?;
        let construct = self.construct(local)?;

        for column in &construct.fields {
            if let Code::Foreign(foreign) = &column.code {
                let _ = foreign.correlate(column)?;
            }
        }

        let check = construct.check()?;
        let flat = match &self.attr.flat {
            Some(_) => Some(construct.flat()?),
            None => None,
        };
        let form = match self.formable() {
            true => Some(construct.form(Source::Column)?),
            false => None,
        };

        let from_row = form.map(|form| quote::quote! {
            #[automatically_derived]
            impl<'r> #krate::sqlx::FromRow<'r, #krate #row> for #ident {
                fn from_row(row: &'r #krate #row) -> #krate::sqlx::Result<Self> {
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
            impl #krate::Table for #ident {
                const AUTOMATICALLY_DERIVED: () = ();
            }
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
                let opt = optype(&ty).and_then(|(op, ty)| optype(ty).map(|_| op));
                if let Some(option) = opt {
                    let option = argone(option);
                    let krate = self.table.krate()?;
                    let skip = quote::quote! { #option::is_none }.to_string();
                    let with = quote::quote! { #krate::double_option }.to_string();
                    fttr = respanned(span, quote::quote! {
                        #[serde(default, with = #with, skip_serializing_if = #skip)]
                        #fttr
                    });
                }
            }
            Ok(quote::quote! { #fttr #vis #ident: #ty })
        }).collect::<Result<Vec<_>>>()?;

        let body = match fields.len() {
            0 => quote::quote! { ; },
            _ => quote::quote! { {
                #(#fields,)*
            } }
        };

        Ok(quote::quote! {
            #derive #attr #vis struct #name #body
        })
    }

}



impl QueryTable {

    pub fn flats(&self) -> Result<Vec<TokenStream>> {
        let mut flats = Vec::new();
        for column in self.coded()? {
            let column = column?;
            if let Code::Query = column.code {
                let ty = self.typed(column.field)?;
                let ident = &column.field.ident;
                flats.push(quote::quote! {
                    #ident: #ty
                });
            }
        }
        Ok(flats)
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
        let row = row![];
        let ident = &self.table.ident;
        let krate = self.table.krate()?;

        let derive = self.table.derive(Structs::Flat)?;
        let flat = self.table.name(Structs::Flat)?;
        let vis = self.table.vis(Structs::Flat)?;
        let flats = self.flats()?;

        let body = match flats.len() {
            0 => quote::quote! { ; },
            _ => quote::quote! { {
                #(#flats,)*
            } }
        };

        let flat_row = match &self.table.attr.flat_row {
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
                Some(quote::quote! {
                    #[automatically_derived]
                    impl<'r> #krate::sqlx::FromRow<'r, #krate #row> for #flat {
                        fn from_row(row: &'r #krate #row) -> #krate::sqlx::Result<Self> {
                            use #krate::sqlx::Row as _;
                            #krate::sqlx::Result::Ok(#flat { #(#sets,)* })
                        }
                    }
                })
            }
        };

        let from_flat = match (&self.table.attr.try_from_flat, &self.table.attr.from_flat) {
            (Some(_), _) => {
                let form = self.form(Source::Field)?;
                Some(quote::quote! {
                    #[automatically_derived]
                    impl ::core::convert::TryFrom<#flat> for #ident {
                        type Error = #krate::sqlx::error::Error;
                        fn try_from(row: #flat) -> ::core::result::Result<Self, Self::Error> {
                            ::core::result::Result::Ok(#form)
                        }
                    }
                })
            }
            (_, Some(_)) => {
                let form = self.form(Source::Field)?;
                Some(quote::quote! {
                    #[automatically_derived]
                    impl ::core::convert::From<#flat> for #ident {
                        fn from(row: #flat) -> Self {
                            #form
                        }
                    }
                })
            }
            _ => None,
        };

        Ok(quote::quote! {
            #[allow(non_snake_case)]
            #derive #vis struct #flat #body
            #flat_row
            #from_flat
            #[automatically_derived]
            impl #krate::Flat for #ident {
                const AUTOMATICALLY_DERIVED: () = ();
                type Flat = #flat;
            }
        })
    }

}



struct Former<'c> {
    target: &'c Constructed<'c>,
    source: &'c Constructed<'c>,
    option: Nullable<'c>,
    gifted: Cow<'c, syn::Ident>,
}

impl Construct<'_> {

    pub fn form(&self, source: Source) -> Result<TokenStream> {
        self.formed(None, source, self)
    }

    fn formed(&self, former: Option<&Former>, source: Source, base: &Construct) -> Result<TokenStream> {
        let fields = self.fields.iter().map(|column| {
            let value = match &column.code {
                Code::Skip => {
                    let span = column.field.attr.skip.spany().unwrap_or_else(Span::call_site);
                    match column.field.attr.default.as_ref().and_then(|data| data.data.as_ref()) {
                        None => quote::quote_spanned!(span => ::core::default::Default::default()),
                        Some(default) => quote::quote_spanned!(span => #default),
                    }
                }
                Code::Query => match source {
                    Source::Field => match (former, column.nullable()?) {
                        (Some(former), None) => {
                            let ident = column.ident()?;
                            let option = former.option.option()?;
                            let arg = quote::quote_spanned!(ident.span() => val);
                            let row = quote::quote_spanned!(ident.span() => row.#ident);
                            let none = former.source.none(&former.option.default()?, base)?;
                            let some = column.from(&arg, base)?;
                            quote::quote! {
                                match #row {
                                    #option::Some(#arg) => #some,
                                    #option::None => break #none,
                                }
                            }
                        }
                        (_, Some(nullable)) if nullable.defaulted() => {
                            let ident = column.ident()?;
                            let option = nullable.option()?;
                            let arg = quote::quote_spanned!(ident.span() => val);
                            let row = quote::quote_spanned!(ident.span() => row.#ident);
                            let none = nullable.default()?;
                            let some = column.from(&arg, base)?;
                            quote::quote! {
                                match #row {
                                    #option::Some(#arg) => #some,
                                    #option::None => #none,
                                }
                            }
                        }
                        _ => {
                            let ident = column.ident()?;
                            let row = quote::quote_spanned!(ident.span() => row.#ident);
                            let from = column.from(&row, base)?;
                            quote::quote! { #from }
                        }
                    }
                    Source::Column => {
                        let ident = column.ident()?;
                        let param = former.and_then(|former| {
                            match ident.eq(&former.gifted) {
                                true => Some(ident),
                                false => None,
                            }
                        });
                        match param {
                            Some(param) => {
                                let arg = quote::quote! { #param };
                                let some = column.from(&arg, base)?;
                                quote::quote! { #some }
                            }
                            None => match column.defaulted()? {
                                Some(defaulted) => {
                                    let ty = column.typed()?;
                                    let alias = column.alias()?;
                                    let option = defaulted.option()?;
                                    let default = defaulted.default()?;
                                    let span = column.field.ident.span();
                                    let arg = quote::quote_spanned!(span => val);
                                    let get = quote::quote_spanned!(span => row.try_get);
                                    let some = column.from(&arg, base)?;
                                    quote::quote! {
                                        match #get::<#ty, _>(#alias)? {
                                            #option::Some(#arg) => #some,
                                            #option::None => #default,
                                        }
                                    }
                                }
                                _ => {
                                    let ty = column.typed()?;
                                    let alias = column.alias()?;
                                    let span = column.field.ident.span();
                                    let get = quote::quote_spanned!(span => row.try_get);
                                    let arg = quote::quote! { #get::<#ty, _>(#alias)? };
                                    let from = column.from(&arg, base)?;
                                    quote::quote! { #from }
                                }
                            }
                        }
                    }
                }
                Code::Foreign(construct) => {
                    let nullable = match construct.nullable()? {
                        None => None,
                        Some(option) => {
                            let target = match construct.constitute()? {
                                Some(field) => field,
                                None => {
                                    let ident = &construct.table.ident;
                                    let span = match column.field.attr.foreign.spany() {
                                        None => column.field.ident.span(),
                                        Some(span) => span,
                                    };
                                    let msg = format!("ambiguous left join on {ident}: \
                                        all fetched fields are nullable");
                                    return Err(syn::Error::new(span, msg));
                                }
                            };
                            let source = &column;
                            let gifted = target.ident()?;
                            Some(Former { option, source, target, gifted })
                        }
                    };
                    let former = nullable.as_ref().or(former);
                    let formed = construct.formed(former, source, base)?;
                    match nullable {
                        None => {
                            let from = column.from(&formed, base)?;
                            quote::quote! { #from }
                        }
                        Some(former) => match source {
                            Source::Field => {
                                let from = column.from(&former.option.some(&formed)?, base)?;
                                quote::quote! { loop { break #from; } }
                            }
                            Source::Column => {
                                let gifted = &former.gifted;
                                let nullable = &former.option;
                                let option = nullable.option()?;
                                let ty = former.target.typed()?;
                                let alias = former.target.alias()?;
                                let some = column.from(&nullable.some(&formed)?, base)?;
                                let none = column.none(&nullable.default()?, base)?;
                                quote::quote! {
                                    match row.try_get::<#option<#ty>, _>(#alias)? {
                                        #option::Some(#gifted) => #some,
                                        #option::None => #none,
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
