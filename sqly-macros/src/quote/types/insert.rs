use super::*;



impl TryFrom<InsertTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: InsertTable) -> Result<TokenStream> {
        cache::store().insert(table)
    }
}

impl Cache for InsertTable {

    fn id(&self) -> Result<Id> {
        Id::try_from(&self.ident)
    }

    fn dep(&self) -> Result<Dep> {
        let mut dep = Dep::new();
        let table = &self.attr.table.data.data;
        dep.art(Key::Table(table.try_into()?));
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}



impl InsertTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let table = &self.attr.table.data.data;
        let query = self.query()?;
        let ident = &self.ident;
        let res = result!['q];

        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::Insert for #ident {
                type Table = #table;
                type Query<'q> = #res;
                fn insert(&self) -> Self::Query<'_> {
                    #query
                }
            }
        })
    }

}



impl InsertTable {

    #[cfg(feature = "postgres")]
    pub fn query(&self) -> Result<TokenStream> {
        let mut query = String::new();
        let mut args = Vec::new();
        let table = self.table()?;

        write!(&mut query,
            "INSERT INTO \"{table}\" (\n"
        ).unwrap();

        let mut i = 1;
        for field in self.fields()? {
            let column = self.column(field)?;
            write!(&mut query,
                "\t\"{column}\",\n"
            ).unwrap();
            i += 1;
        }
        let trunc = if i > 1 { 2 } else { 0 };
        query.truncate(query.len() - trunc);
        query.push_str("\n)\nVALUES\n\t(");

        let mut i = 1;
        for field in self.fields()? {
            let value = self.value(field)?;
            write!(&mut query,
                "${i}, "
            ).unwrap();
            args.push(value);
            i += 1;
        }
        let trunc = if i > 1 { 2 } else { 0 };
        query.truncate(query.len() - trunc);
        query.push(')');

        self.print(&query, &args)?;

        Ok(fun!(query, args))
    }

}
