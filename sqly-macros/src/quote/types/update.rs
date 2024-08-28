use super::*;



impl TryFrom<UpdateTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: UpdateTable) -> Result<TokenStream> {
        table.debug(table.derived()?)
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
        let table = &self.attr.table_name.data.data;
        let mut query = String::new();
        let mut args = Vec::new();

        query.push_str(&format!(
            "UPDATE \"{table}\" SET\n",
        ));

        let mut i = 1;
        for field in self.fields()? {
            if field.attr.key.is_none() {
                let column = self.column(field)?;
                let value = self.value(field)?;
                query.push_str(&format!(
                    "\t\"{column}\" = ${i},\n"
                ));
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
                query.push_str(&format!(
                    "\t\"{column}\" = ${j} AND\n"
                ));
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
