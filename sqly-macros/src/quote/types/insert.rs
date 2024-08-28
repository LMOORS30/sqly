use super::*;



impl TryFrom<InsertTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: InsertTable) -> Result<TokenStream> {
        table.debug(table.derived()?)
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
        let table = &self.attr.table_name.data.data;
        let mut query = String::new();
        let mut args = Vec::new();

        query.push_str(&format!(
            "INSERT INTO \"{table}\" (\n"
        ));

        let mut i = 1;
        for field in self.fields()? {
            let column = self.column(field)?;
            query.push_str(&format!(
                "\t\"{column}\",\n"
            ));
            i += 1;
        }
        let trunc = if i > 1 { 2 } else { 0 };
        query.truncate(query.len() - trunc);
        query.push_str("\n)\nVALUES\n\t(");

        let mut i = 1;
        for field in self.fields()? {
            let value = self.value(field)?;
            query.push_str(&format!(
                "${i}, "
            ));
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
