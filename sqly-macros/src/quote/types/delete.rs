use super::*;



impl TryFrom<DeleteTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: DeleteTable) -> Result<TokenStream> {
        table.debug(table.derived()?)
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
        let table = &self.attr.table_name.data.data;
        let mut query = String::new();
        let mut args = Vec::new();

        query.push_str(&format!(
            "DELETE FROM \"{table}\"\nWHERE\n"
        ));

        let mut i = 1;
        for field in self.fields()? {
            let column = self.column(field)?;
            let value = self.value(field)?;
            query.push_str(&format!(
                "\t\"{column}\" = ${i} AND\n"
            ));
            args.push(value);
            i += 1;
        }
        let trunc = if i > 1 { 5 } else { 7 };
        query.truncate(query.len() - trunc);

        self.print(&query, &args)?;

        Ok(fun!(query, args))
    }

}


