use super::*;



impl TryFrom<UpdateTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: UpdateTable) -> Result<TokenStream> {
        cache::store().update(table)
    }
}

impl Cache for UpdateTable {

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



impl UpdateTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let table = &self.attr.table.data.data;
        let query = self.query()?;
        let ident = &self.ident;
        let res = result!['q];

        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::Update for #ident {
                type Table = #table;
                type Query<'q> = #res;
                fn update(&self) -> Self::Query<'_> {
                    #query
                }
            }
        })
    }

}



impl UpdateTable {

    #[cfg(feature = "postgres")]
    pub fn query(&self) -> Result<TokenStream> {
        let mut query = String::new();
        let mut args = Vec::new();
        let table = self.table()?;

        write!(&mut query,
            "UPDATE \"{table}\" SET\n",
        ).unwrap();

        let mut i = 1;
        for field in self.fields()? {
            if field.attr.key.is_none() {
                let column = self.column(field)?;
                let value = self.value(field)?;
                write!(&mut query,
                    "\t\"{column}\" = ${i},\n"
                ).unwrap();
                args.push(value);
                i += 1;
            }
        }
        let trunc = if i > 1 { 2 } else { 5 };
        query.truncate(query.len() - trunc);
        query.push_str("\nWHERE\n");

        let mut j = i;
        for field in &self.fields {
            if field.attr.key.is_some() {
                let column = self.column(field)?;
                let value = self.value(field)?;
                write!(&mut query,
                    "\t\"{column}\" = ${j} AND\n"
                ).unwrap();
                args.push(value);
                j += 1;
            }
        }
        let trunc = if j > i { 5 } else { 7 };
        query.truncate(query.len() - trunc);

        self.print(&query, &args)?;

        Ok(fun!(query, args))
    }

}
