use super::*;



impl TryFrom<DeleteTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: DeleteTable) -> Result<TokenStream> {
        cache::store().delete(table)
    }
}

impl Cache for DeleteTable {

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

impl DeleteTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let table = &self.attr.table.data.data;
        let query = self.query()?;
        let ident = &self.ident;
        let res = result!['q];

        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::Delete for #ident {
                type Table = #table;
                type Query<'q> = #res;
                fn delete(&self) -> Self::Query<'_> {
                    #query
                }
            }
        })
    }

}



impl DeleteTable {

    #[cfg(feature = "postgres")]
    pub fn query(&self) -> Result<TokenStream> {
        let mut query = String::new();
        let mut args = Vec::new();
        let table = self.table()?;

        write!(&mut query,
            "DELETE FROM \"{table}\"\nWHERE\n"
        ).unwrap();

        let mut i = 1;
        for field in self.fields()? {
            let column = self.column(field, Target::Query)?;
            let value = self.value(field, Target::Query)?;
            write!(&mut query,
                "\t\"{column}\" = ${i} AND\n"
            ).unwrap();
            args.push(value);
            i += 1;
        }
        let trunc = if i > 1 { 5 } else { 7 };
        query.truncate(query.len() - trunc);

        self.print(&query, &args)?;
        Ok(fun!(query, args))
    }

}


