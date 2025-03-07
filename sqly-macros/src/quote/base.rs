use super::*;



impl QueryTable {

    pub fn attrs(&self, r#type: Types) -> Result<Vec<TokenStream>> {
        let ident = &self.ident;
        let mut attrs = vectok![
            quote::quote! { table = #ident },
            self.attr.rename,
            self.attr.unchecked,
            self.attr.print,
            self.attr.debug,
        ];
        let a = &self.attr;
        match r#type {
            Types::Delete => args!(attrs, [
                (filter = a.delete_filter, a.filter),
            ]),
            Types::Insert => {},
            Types::Select => args!(attrs, [
                (filter = a.select_filter, a.filter),
            ]),
            Types::Update => args!(attrs, [
                (filter = a.update_filter, a.filter),
            ]),
        }
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

    pub fn derive(&self, r#type: Structs) -> Result<TokenStream> {
        let derives = self.derives(r#type)?;
        let span = match r#type {
            Structs::Flat => self.attr.flat.as_ref(),
            Structs::Delete => self.attr.delete.as_ref(),
            Structs::Insert => self.attr.insert.as_ref(),
            Structs::Select => self.attr.select.as_ref(),
            Structs::Update => self.attr.update.as_ref(),
        }.map(|attr| attr.span).unwrap_or_else(|| {
            Span::call_site()
        });
        let derive = match r#type {
            Structs::Delete => quote::quote_spanned! { span => ::sqly::Delete },
            Structs::Insert => quote::quote_spanned! { span => ::sqly::Insert },
            Structs::Select => quote::quote_spanned! { span => ::sqly::Select },
            Structs::Update => quote::quote_spanned! { span => ::sqly::Update },
            Structs::Flat => return Ok((!derives.is_empty()).then(|| {
                quote::quote! { #[derive(#(#derives,)*)] }
            }).unwrap_or_default()),
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
        if let Types::Update = r#type {
            if self.keyed(field, r#type) {
                let span = field.attr.key.as_ref().map(|key| {
                    key.data.iter().find(|val| {
                        r#type == val.data.into()
                    }).map_or(key.span, |val| val.span)
                }).unwrap_or_else(|| Span::call_site());
                let key = quote::quote_spanned! { span => key };
                fttrs.push(key);
            }
        }
        let a = &field.attr;
        match r#type {
            Types::Delete => args!(fttrs, [
                (filter = a.delete_filter, a.filter),
            ]),
            Types::Insert => args!(fttrs, [
                (insert = field.attr.insert),
            ]),
            Types::Select => args!(fttrs, [
                (filter = a.select_filter, a.filter),
            ]),
            Types::Update => args!(fttrs, [
                (filter = a.update_filter, a.filter),
                (update = field.attr.update),
            ]),
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



impl QueryTable {

    pub fn typed(&self, field: &QueryField) -> Result<TokenStream> {
        let ty = self.ty(field)?;
        let typed = match self.defaulted(field)? {
            Some(nullable) => {
                let option = nullable.option()?;
                quote::quote! { #option<#ty> }
            }
            _ => quote::quote! { #ty },
        };
        Ok(typed)
    }

}



impl Constructed<'_> {

    pub fn from(&self) -> Result<TokenStream> {
        let from = match &self.field.attr.from {
            None => TokenStream::new(),
            Some(_) => {
                let ty = &self.field.ty;
                quote::quote! { <#ty>::from }
            }
        };
        Ok(from)
    }

    pub fn none(&self) -> Result<TokenStream> {
        let none = match &self.field.attr.default {
            Some(_) => TokenStream::new(),
            None => self.from()?,
        };
        Ok(none)
    }

}



impl Nullable<'_> {

    pub fn some(&self) -> Result<TokenStream> {
        let some = match self {
            Nullable::Option(path) => {
                let path = argone(path);
                quote::quote! { #path::Some }
            }
            Nullable::Default(_) => {
                TokenStream::new()
            }
        };
        Ok(some)
    }

    pub fn option(&self) -> Result<TokenStream> {
        let option = match self {
            Nullable::Option(path) => {
                let path = argone(path);
                quote::quote! { #path }
            }
            Nullable::Default(_) => {
                quote::quote! { ::core::option::Option }
            }
        };
        Ok(option)
    }

    pub fn default(&self) -> Result<TokenStream> {
        let default = match self {
            Nullable::Option(path) => {
                let path = argone(path);
                quote::quote! { #path::None }
            }
            Nullable::Default(path) => match &path {
                Some(path) => quote::quote! { #path() },
                None => quote::quote! { ::core::default::Default::default() },
            }
        };
        Ok(default)
    }

}



impl Construct<'_> {

    pub fn check(&self) -> Result<TokenStream> {
        if !self.table.checked() {
            return Ok(TokenStream::new());
        }

        let mut fields = Vec::new();
        let name = &self.table.ident;
        for column in self.fields.iter() {
            if let Code::Query = column.code {
                let ty = column.typed()?;
                let ident = column.ident()?;
                fields.push(quote::quote! {
                    #ident: #ty
                });
            }
        }

        if fields.is_empty() { return Ok(TokenStream::new()); }
        let query = self.query(Target::Macro, Scope::Local)?;
        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::Check for #name {
                #[allow(unused)]
                fn check(&self) {
                    #[allow(non_snake_case)]
                    struct #name { #(#fields,)* }
                    ::sqlx::query_as!(#name, #query);
                }
            }
        })
    }

}



macro_rules! base {
($table:ty, $field:ty, $upper:ident, $lower:ident) => {
both!($table, $field);

paste::paste! {

impl $table {

    pub fn checking<F>(&self, args: &[&$field], cb: F) -> Result<TokenStream>
    where F: FnOnce(&[TokenStream]) -> Result<TokenStream> {
        if !self.checked() {
            return Ok(TokenStream::new());
        }
        let obj = &self.ident;
        let args = args.iter().map(|field| {
            self.value(field, Target::Macro)
        }).collect::<Result<Vec<_>>>()?;
        let check = cb(&args)?;
        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::[<$upper Check>] for #obj {
                #[allow(unused)]
                fn [<$lower _check>](&self) {
                    #check
                }
            }
        })
    }

    pub fn check(&self, query: &str, args: &[&$field]) -> Result<TokenStream> {
        self.checking(args, |args| Ok(quote::quote! {
            ::sqlx::query!(#query #(, #args)*);
        }))
    }

    pub fn blanket(&self) -> Result<TokenStream> {
        let obj = &self.ident;
        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::$upper for #obj {
                type Table = <Self as ::sqly::[<$upper Impl>]>::Table;
                type Query<'a> = <Self as ::sqly::[<$upper Impl>]>::Query<'static, 'a>
                    where Self: 'a;
                fn $lower(&self) -> Self::Query<'_> {
                    <Self as ::sqly::[<$upper Impl>]>::[<$lower _from>](
                        <Self as ::sqly::[<$upper Impl>]>::[<$lower _sql>](self)
                    )
                }
            }
        })
    }

    pub fn [<$lower>](&self, string: &str, args: &[&$field], map: Option<&syn::Path>) -> Result<TokenStream> {
        let db = db![];
        let obj = &self.ident;
        let res = quote::quote! { ::core::result::Result };
        let err = quote::quote! { ::sqlx::error::BoxDynError };
        let row = quote::quote! { <#db as ::sqlx::Database>::Row };
        let arg = quote::quote! { <#db as ::sqlx::Database>::Arguments };

        let typle = match &self.attr.table.data.data {
            Paved::String(_) => quote::quote! { &'static str },
            Paved::Path(path) => quote::quote! { #path },
        };
        let query = match map {
            None => quote::quote! { ::sqlx::query::Query<'q, #db, #arg<'a>> },
            Some(path) => quote::quote! {
                ::sqlx::query::Map<'q, #db, fn(#row) -> ::sqlx::Result<#path>, #arg<'a>>
            }
        };
        let from = match args.len() {
            0 => quote::quote! { &'q str },
            _ => quote::quote! { (&'q str, #res<#arg<'a>, #err>) }
        };
        let sql = match args.len() {
            0 => quote::quote! { &'static str },
            _ => quote::quote! { (&'static str, #res<#arg<'a>, #err>) }
        };
        let with = match args.len() {
            0 => quote::quote! { query, #res::Ok(#arg::default()) },
            _ => quote::quote! { query.0, query.1 },
        };
        let map = map.map(|path| quote::quote! {
            .try_map(<#path as ::sqly::Table>::from_row)
        });

        let len = args.len();
        let bind = (0..len).map(|i| {
            let i = syn::Index::from(i);
            quote::quote! { arg.#i }
        }).collect::<Vec<_>>();
        let expr = args.iter().map(|field| {
            self.value(field, Target::Function)
        }).collect::<Result<Vec<_>>>()?;
        let run = (!args.is_empty()).then(|| quote::quote! {
                let arg = (#(&(#expr),)*);
                use ::sqlx::Arguments as _;
                let mut args = #arg::default();
                args.reserve(#len, 0 #(+ ::sqlx::Encode::<#db>::size_hint(#bind))*);
                let args = #res::Ok(args)
                #(.and_then(move |mut args| args.add(#bind).map(move |()| args)))*;
                (#string, args)
        }).unwrap_or_else(|| quote::quote! { #string });

        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::[<$upper Impl>] for #obj {
                type Table = #typle;
                type Query<'q, 'a> = #query;
                type From<'q, 'a> = #from;
                type Sql<'a> = #sql;
                fn [<$lower _sql>](&self) -> Self::Sql<'_> { #run }
                fn [<$lower _from>]<'q, 'a>(query: Self::From<'q, 'a>) -> Self::Query<'q, 'a> {
                    ::sqlx::__query_with_result(#with)#map
                }
            }
        })
    }

}

} } }



macro_rules! both {
($table:ty, $field:ty) => {

impl $table {

    pub fn value(&self, field: &$field, target: Target) -> Result<TokenStream> {
        let value = match &field.attr.value {
            Some(value) => {
                let expr = &value.data.data;
                let span = value.data.span;
                let unfer = unfer(expr);
                let unfer = unfer.as_ref().unwrap_or(expr);
                match (target, &field.attr.infer) {
                    (Target::Macro, None) => quote::quote_spanned!(span => #expr),
                    (Target::Macro, Some(_)) => quote::quote_spanned!(span => (#unfer) as _),
                    (Target::Function, _) => quote::quote_spanned!(span => #unfer),
                }
            }
            None => {
                let ident = &field.ident;
                let span = field.ty.span();
                match &field.attr.infer.as_ref().map(|_| target) {
                    Some(Target::Macro) => quote::quote_spanned!(span => self.#ident as _),
                    Some(Target::Function) | None => quote::quote_spanned!(span => self.#ident),
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
                }
                Err(err) => {
                    let rs = res.to_string();
                    println!("{}", rs);
                    Err(err)
                }
            }
            None => Ok(res),
        }
    }

}

} }



both!(QueryTable, QueryField);
base!(DeleteTable, DeleteField, Delete, delete);
base!(InsertTable, InsertField, Insert, insert);
base!(SelectTable, SelectField, Select, select);
base!(UpdateTable, UpdateField, Update, update);
