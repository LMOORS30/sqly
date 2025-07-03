use super::*;



impl QueryTable {

    pub fn attrs(&self, r#type: Types) -> Result<Vec<TokenStream>> {
        let ident = &self.ident;
        let mut attrs = vectok![
            quote::quote! { table = #ident },
            self.attr.rename_all,
            self.attr.krate,
            self.attr.unchecked,
            self.attr.print,
            self.attr.debug,
        ];
        if let Some(dynamic) = &self.attr.dynamic {
            for field in &self.fields {
                if self.fielded(field, r#type) {
                    if self.optional(field, r#type).is_some() {
                        attrs.push(quote::quote! { #dynamic });
                        break;
                    }
                }
            }
        }
        let a = &self.attr;
        match r#type {
            Types::Delete => args!(attrs, [
                (filter = a.delete_filter, a.filter),
                (returning = a.delete_returning, a.returning),
            ]),
            Types::Insert => args!(attrs, [
                (returning = a.insert_returning, a.returning),
            ]),
            Types::Select => args!(attrs, [
                (filter = a.select_filter, a.filter),
            ]),
            Types::Update => args!(attrs, [
                (filter = a.update_filter, a.filter),
                (returning = a.update_returning, a.returning),
            ]),
        }
        Ok(attrs)
    }

    pub fn attr(&self, r#type: Types) -> Result<TokenStream> {
        let attrs = self.attrs(r#type)?;
        let attr = match attrs.len() {
            0 => TokenStream::new(),
            _ => quote::quote! { #[sqly(#(#attrs),*)] }
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
        let krate = self.krate()?;
        let derive = respanned(span, match r#type {
            Structs::Delete => quote::quote! { #krate::Delete },
            Structs::Insert => quote::quote! { #krate::Insert },
            Structs::Select => quote::quote! { #krate::Select },
            Structs::Update => quote::quote! { #krate::Update },
            Structs::Flat => return Ok((!derives.is_empty()).then(|| {
                quote::quote! { #[derive(#(#derives,)*)] }
            }).unwrap_or_default()),
        });
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
                    }).map_or(key.span, |val| val.span())
                }).unwrap_or_else(|| Span::call_site());
                let key = quote::quote_spanned!(span => key);
                fttrs.push(key);
            }
        }
        if let Some(span) = self.optional(field, r#type) {
            fttrs.push(quote::quote_spanned!(span => optional));
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
            _ => quote::quote! { #[sqly(#(#fttrs),*)] }
        };
        Ok(fttr)
    }

}



impl QueryTable {

    pub fn typed<'c>(&'c self, field: &'c QueryField) -> Result<Cow<'c, syn::Type>> {
        let ty = self.ty(field)?;
        let typed = match self.defaulted(field)? {
            Some(nullable) => {
                let option = nullable.option()?;
                Cow::Owned(syn::parse_quote! { #option<#ty> })
            }
            None => Cow::Borrowed(ty),
        };
        Ok(typed)
    }

}

impl<'c, T: Struct> Scalar<'c, T> {

    pub fn typed(&self) -> Result<Cow<'c, syn::Type>> {
        let typed = match self {
            Scalar::Table(_, field) => Cow::Borrowed(field.ty()),
            Scalar::Paved(table, field) => table.typed(field)?,
        };
        Ok(typed)
    }

}



impl Constructed<'_> {

    pub fn from(&self, arg: &TokenStream, base: &Construct) -> Result<TokenStream> {
        let ty = &self.field.ty;
        let from = match (&self.field.attr.try_from, &self.field.attr.from) {
            (Some(name), _) => {
                if let Some(span) = base.table.attr.from_flat.spany() {
                    let msg = "conflicting attributes: \
                        #[sqly(from_flat)] with #[sqly(try_from)] on foreign fields\n\
                        help: use #[sqly(try_from_flat)] instead";
                    return Err(syn::Error::new(span, msg));
                }
                let column = self.alias()?;
                let krate = self.table.krate()?;
                quote::quote_spanned!{name.span=>
                    <#ty as ::core::convert::TryFrom<_>>::try_from(#arg).map_err(|e| {
                        #krate::sqlx::Error::ColumnDecode {
                            source: #krate::__spec_error!(e),
                            index: #column.to_string(),
                        }
                    })?
                }
            },
            (_, Some(name)) => quote::quote_spanned!{name.span=>
                <#ty as ::core::convert::From<_>>::from(#arg)
            },
            _ => arg.clone(),
        };
        Ok(from)
    }

    pub fn none(&self, arg: &TokenStream, base: &Construct) -> Result<TokenStream> {
        let none = match &self.field.attr.default {
            None => self.from(arg, base)?,
            Some(_) => arg.clone(),
        };
        Ok(none)
    }

}

impl Nullable<'_> {

    pub fn some(&self, arg: &TokenStream) -> Result<TokenStream> {
        let some = match self {
            Nullable::Option(path) => {
                let path = argone(path);
                let span = path.spanned();
                quote::quote_spanned!(span => #path::Some(#arg))
            }
            Nullable::Default(_, _) => arg.clone(),
        };
        Ok(some)
    }

    pub fn option(&self) -> Result<TokenStream> {
        let option = match self {
            Nullable::Option(path) => {
                let path = argone(path);
                quote::quote! { #path }
            }
            Nullable::Default(span, _) => {
                quote::quote_spanned!(*span => ::core::option::Option)
            }
        };
        Ok(option)
    }

    pub fn default(&self) -> Result<TokenStream> {
        let default = match self {
            Nullable::Option(path) => {
                let path = argone(path);
                let span = path.spanned();
                quote::quote_spanned!(span => #path::None)
            }
            Nullable::Default(span, expr) => match &expr {
                Some(default) => quote::quote! { #default },
                None => quote::quote_spanned!(*span => ::core::default::Default::default()),
            }
        };
        Ok(default)
    }

}



impl Construct<'_> {

    pub fn check(&self) -> Result<TokenStream> {
        if self.table.unchecked() {
            return Ok(TokenStream::new());
        }
        let obj = &self.table.ident;
        let krate = self.table.krate()?;
        let fields = self.table.flats()?;
        let query = self.table.query(Target::Macro)?;
        let call = match self.table.untyped() {
            false => quote::quote! { query_as },
            true => quote::quote! { query_as_unchecked },
        };
        Ok(quote::quote! {
            #[automatically_derived]
            impl #krate::Check for #obj {
                #[allow(unused)]
                fn check(&self) -> ! {
                    struct #obj { #(#fields,)* }
                    #krate::sqlx::#call!(#obj, #query);
                    ::core::panic!()
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

    pub fn [<$lower>](&self, done: &Done<$table>, returns: &Returns<$table>) -> Result<TokenStream> {
        let db = db![];
        let row = row![];
        let obj = &self.ident;
        let krate = self.krate()?;
        let res = quote::quote! { ::core::result::Result };
        let err = quote::quote! { #krate::sqlx::error::BoxDynError };
        let arg = quote::quote! { <#krate #db as #krate::sqlx::Database>::Arguments };
        let map = quote::quote! { fn(#krate #row) -> #krate::sqlx::Result };

        let typle = match &self.attr.table.data.data {
            Paved::String(_) => quote::quote! { &'static str },
            Paved::Path(path) => quote::quote! { #path },
        };
        let query = match returns {
            Returns::None => quote::quote! {
                #krate::sqlx::query::Query<'q, #krate #db, #arg<'a>>
            },
            Returns::Scalar(item) => {
                let scalar = item.typed()?;
                quote::quote! { #krate::sqlx::query::Map<'q, #krate #db, #map<#scalar>, #arg<'a>> }
            },
            Returns::Tuple(list) => {
                let tuple = list.iter().map(|item| item.typed()).collect::<Result<Vec<_>>>()?;
                quote::quote! { #krate::sqlx::query::Map<'q, #krate #db, #map<(#(#tuple,)*)>, #arg<'a>> }
            },
            Returns::Table(path, _) | Returns::Construct(path, _) => quote::quote! {
                #krate::sqlx::query::Map<'q, #krate #db, #map<#path>, #arg<'a>>
            },
        };
        let sql = match self.dynamic() {
            Some(_) => quote::quote! { String },
            None => quote::quote! { &'static str },
        };
        let sql = match done.args.len() {
            0 => quote::quote! { #sql },
            _ => quote::quote! { (#sql, #res<#arg<'a>, #err>) },
        };
        let sql = match self.certain() {
            false => quote::quote! { ::core::option::Option<#sql> },
            true => quote::quote! { #sql },
        };
        let from = match done.args.len() {
            0 => quote::quote! { &'q str },
            _ => quote::quote! { (&'q str, #res<#arg<'a>, #err>) },
        };
        let with = match done.args.len() {
            0 => quote::quote! { query, #res::Ok(#arg::default()) },
            _ => quote::quote! { query.0, query.1 },
        };
        let gun = match returns {
            Returns::None => quote::quote! {
                #krate::sqlx::__query_with_result(#with)
            },
            Returns::Scalar(_) => quote::quote! {
                #krate::sqlx::__query_with_result(#with).try_map(|row| {
                    <(_,) as #krate::sqlx::FromRow<_>>::from_row(&row).map(|row| row.0)
                })
            },
            Returns::Tuple(_) => quote::quote! {
                #krate::sqlx::__query_with_result(#with).try_map(|row| {
                    #krate::sqlx::FromRow::from_row(&row)
                })
            },
            Returns::Table(_, _) | Returns::Construct(_, _) => quote::quote! {
                #krate::sqlx::__query_with_result(#with).try_map(|row| {
                    #krate::sqlx::FromRow::from_row(&row)
                })
            },
        };
        let run = &done.stream;

        Ok(quote::quote! {
            #[automatically_derived]
            impl #krate::[<$upper Impl>] for #obj {
                type Table = #typle;
                type Query<'q, 'a> = #query;
                type From<'q, 'a> = #from;
                type Sql<'a> = #sql;
                fn [<$lower _sql>](&self) -> Self::Sql<'_> { #run }
                fn [<$lower _from>]<'q, 'a>(query: Self::From<'q, 'a>) -> Self::Query<'q, 'a> { #gun }
            }
        })
    }

    pub fn blanket(&self) -> Result<TokenStream> {
        if self.dynamic().is_some() {
            return Ok(TokenStream::new());
        }
        let obj = &self.ident;
        let krate = self.krate()?;
        Ok(quote::quote! {
            #[automatically_derived]
            impl #krate::$upper for #obj {
                type Table = <Self as #krate::[<$upper Impl>]>::Table;
                type Query<'a> = <Self as #krate::[<$upper Impl>]>::Query<'static, 'a>
                    where Self: 'a;
                fn $lower(&self) -> Self::Query<'_> {
                    <Self as #krate::[<$upper Impl>]>::[<$lower _from>](
                        <Self as #krate::[<$upper Impl>]>::[<$lower _sql>](self)
                    )
                }
            }
        })
    }

    pub fn check(&self, done: &Done<$table>, returns: &Returns<$table>) -> Result<TokenStream> {
        if self.unchecked() {
            return Ok(TokenStream::new());
        }
        let query = match &done.check {
            Some(check) => check,
            None => {
                let msg = "failed to generate compile time check";
                return Err(syn::Error::new(Span::call_site(), msg));
            }
        };

        let obj = &self.ident;
        let krate = self.krate()?;
        let model = quote::format_ident!("_{obj}");
        let args = done.args.iter().map(|field| {
            self.value(field, Target::Macro)
        }).collect::<Result<Vec<_>>>()?;
        let rip = self.dynamic().map(|_| {
            quote::quote! { use #krate::dynamic::Rip as _; }
        });

        let check = match returns {
            Returns::None => None,
            Returns::Scalar(item) => {
                let ty = item.typed()?;
                let ident = item.ident()?;
                let field = quote::quote! { #ident: #ty };
                Some(Left(vec![field]))
            },
            Returns::Tuple(list) => {
                let fields = list.iter().map(|item| {
                    let ty = item.typed()?;
                    let ident = item.ident()?;
                    Ok(quote::quote! { #ident: #ty })
                }).collect::<Result<Vec<_>>>()?;
                Some(Left(fields))
            },
            Returns::Table(path, table) => match &table.attr.flat {
                None => Some(Left(table.flats()?)),
                Some(_) => Some(Right(path)),
            }
            Returns::Construct(path, construct) => match &construct.table.attr.flat {
                None => Some(Left(construct.flats()?)),
                Some(_) => Some(Right(path)),
            }
        };
        let call = match check {
            None => match self.untyped() {
                false => quote::quote! { query },
                true => quote::quote! { query_unchecked },
            }
            Some(_) => match self.untyped() {
                false => quote::quote! { query_as },
                true => quote::quote! { query_as_unchecked },
            }
        };
        let check = match check {
            None => quote::quote! {
                #krate::sqlx::#call!(#query #(, #args)*);
            },
            Some(Left(fields)) => quote::quote! {
                struct #model { #(#fields),* }
                #krate::sqlx::#call!(#model, #query #(, #args)*);
            },
            Some(Right(flat)) => quote::quote! {
                type #model = <#flat as #krate::Flat>::Flat;
                #krate::sqlx::#call!(#model, #query #(, #args)*);
            },
        };

        Ok(quote::quote! {
            #[automatically_derived]
            impl #krate::[<$upper Check>] for #obj {
                #[allow(unused)]
                fn [<$lower _check>](&self) -> ! {
                    #rip
                    #check
                    ::core::panic!()
                }
            }
        })
    }

    pub fn value(&self, field: &$field, target: Target) -> Result<TokenStream> {
        let rip = self.optional(field).map(|span| quote::quote_spanned!(span => .rip()));
        let value = match &field.attr.value {
            Some(value) => {
                let span = value.data.span();
                let expr = &value.data.data;
                let unfer = unfer(expr);
                let unfer = unfer.as_ref().unwrap_or(expr);
                match (target, &field.attr.infer) {
                    (Target::Macro, Some(_)) => quote::quote_spanned!(span => (#unfer) #rip as _),
                    (Target::Macro, None) => quote::quote_spanned!(span => (#expr) #rip),
                    (Target::Function, _) => quote::quote_spanned!(span => #unfer),
                }
            }
            None => {
                let ident = &field.ident;
                let span = field.ty.spanned();
                match (target, &field.attr.infer) {
                    (Target::Macro, Some(_)) => quote::quote_spanned!(span => self.#ident #rip as _),
                    (Target::Macro, None) => quote::quote_spanned!(span => self.#ident #rip),
                    (Target::Function, _) => quote::quote_spanned!(span => self.#ident),
                }
            }
        };
        Ok(value)
    }

    pub fn print(&self, done: &Done<$table>)  -> Result<TokenStream> {
        let print = match &self.attr.print {
            None => return Ok(TokenStream::new()),
            Some(print) => print,
        };
        let query = match done.query.as_ref().or(done.check.as_ref()) {
            Some(query) => {
                let mut tabs = String::new();
                for line in query.split('\n') {
                    tabs.push_str("\n\t");
                    tabs.push_str(line);
                }
                if !tabs.is_empty() {
                    tabs.push_str("\n\t");
                }
                tabs
            }
            None => {
                let debug = print.rename("debug");
                let debug = quote::quote!{ #debug }.to_string();
                format!("no static query generated, use #[sqly({debug})] instead")
            }
        };
        let target = match &done.check {
            Some(_) => Target::Macro,
            None => Target::Function,
        };
        let mut args = String::new();
        for arg in &done.args {
            args.push_str(",\n\t");
            let val = self.value(arg, target)?;
            args.push_str(&val.to_string())
        }
        for arg in &done.rest {
            if self.optional(arg).is_some() {
                args.push_str(",\n\t// ");
                let val = self.value(arg, target)?;
                args.push_str(&val.to_string());
            }
        }
        let sql = format!(
            "{}::query!(\n\tr#\"{}\"#{}\n)",
            self.ident, query, args
        );
        Ok(print.output(&sql))
    }

}

} } }



macro_rules! both {
($table:ty, $field:ty) => {

impl $table {

    pub fn krate(&self) -> Result<Cow<syn::Path>> {
        let krate = match &self.attr.krate {
            Some(krate) => Cow::Borrowed(&krate.data.data),
            None => Cow::Owned(syn::parse_quote! { ::sqly }),
        };
        Ok(krate)
    }

    pub fn debug(&self, mut res: TokenStream) -> Result<TokenStream> {
        let debug = match &self.attr.debug {
            None => return Ok(res),
            Some(debug) => debug,
        };
        match syn::parse2(res.clone()) {
            Ok(tree) => {
                let rs = prettyplease::unparse(&tree);
                res.extend(debug.output(&rs));
                Ok(res)
            }
            Err(_) => {
                let rs = res.to_string();
                res.extend(debug.output(&rs));
                Ok(res)
            }
        }
    }

}

} }



both!(QueryTable, QueryField);
base!(DeleteTable, DeleteField, Delete, delete);
base!(InsertTable, InsertField, Insert, insert);
base!(SelectTable, SelectField, Select, select);
base!(UpdateTable, UpdateField, Update, update);
