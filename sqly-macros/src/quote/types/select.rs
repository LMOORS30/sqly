use super::*;



impl TryFrom<SelectTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: SelectTable) -> Result<TokenStream> {
        cache::store().select(table)
    }
}

impl Cache for SelectTable {

    fn id(&self) -> Result<Id> {
        Id::try_from(&self.ident)
    }

    fn dep(&self) -> Result<Dep> {
        let mut dep = Dep::new();
        let table = &self.attr.table.data.data;
        dep.end(Key::Table(table.try_into()?));
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}

impl SelectTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let table = &self.attr.table.data.data;
        let query = self.query()?;
        let ident = &self.ident;
        let res = result!['q, table];

        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::Select for #ident {
                type Table = #table;
                type Query<'q> = #res;
                fn select(&self) -> Self::Query<'_> {
                    #query
                }
            }
        })
    }

}



impl SelectTable {

    #[cfg(feature = "postgres")]
    pub fn query(&self) -> Result<TokenStream> {
        let res = &self.attr.table.data.data;
        let table = cache::fetch().table(&res.try_into()?)?.sync()?;

        let mut query = table.select(Target::Query)?;
        let args = self.filter(&mut query, Target::Query)?;

        self.print(&query, &args)?;
        let run = fun!(query, args);
        let check = self.check(&table)?;

        Ok(quote::quote! {
            #check
            #run.try_map(<#res as ::sqly::Table>::from_row)
        })
    }

    #[cfg(not(feature = "unchecked"))]
    pub fn check(&self, table: &QueryTable) -> Result<TokenStream> {
        let mut query = table.select(Target::Check)?;
        let args = self.filter(&mut query, Target::Check)?;

        let item = &self.ident;
        let ident = &table.ident;
        let columns = table.columns()?;

        let typed = columns.map(|field| {
            table.field(field, Target::Check)
        });

        let typed = typed.collect::<Result<Vec<_>>>()?;

        Ok(quote::quote! {
            fn __(item: &#item) {
                #[allow(non_snake_case)]
                struct #ident { #(#typed,)* }
                ::sqlx::query_as!(#ident, #query #(, #args)*);
            }
        })
    }

    #[cfg(feature = "unchecked")]
    pub fn check(&self, _: &QueryTable) -> Result<TokenStream> {
        Ok(TokenStream::new())
    }

}



impl QueryTable {

    pub fn select(&self, target: Target) -> Result<String> {
        let table = &self.attr.table.data.data;
        let mut query = String::new();

        write!(&mut query,
            "SELECT\n"
        ).unwrap();

        let mut i = 1;
        for field in self.columns()? {
            let column = self.column(field, Target::Query)?;
            write!(&mut query,
                "\t\"{column}\""
            ).unwrap();
            if let Target::Check = target {
                let column = self.column(field, Target::Check)?;
                write!(&mut query,
                    " AS \"{column}\""
                ).unwrap();
            }
            query.push_str(",\n");
            i += 1;
        }
        let trunc = if i > 1 { 2 } else { 1 };
        query.truncate(query.len() - trunc);

        write!(&mut query,
            "\nFROM \"{table}\""
        ).unwrap();

        Ok(query)
    }

}

impl SelectTable {

    pub fn filter(&self, query: &mut String, target: Target) -> Result<Vec<TokenStream>> {
        let mut args = Vec::new();

        write!(query,
            "\nWHERE\n"
        ).unwrap();

        let mut i = 1;
        for field in self.fields()? {
            let column = self.column(field, Target::Query)?;
            let value = self.value(field, target)?;
            write!(query,
                "\t\"{column}\" = ${i} AND\n"
            ).unwrap();
            args.push(value);
            i += 1;
        }
        let trunc = if i > 1 { 5 } else { 7 };
        query.truncate(query.len() - trunc);

        Ok(args)
    }

}
