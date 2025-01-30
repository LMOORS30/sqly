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
        let (check, query) = self.query()?;
        let ident = &self.ident;
        let res = result!['q];

        Ok(quote::quote! {
            #check
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

    pub fn query(&self) -> Result<(TokenStream, TokenStream)> {
        let mut params = Params::default();
        let mut query = String::new();
        let table = self.table()?;

        let fields = self.cells(&mut params, |field| {
            Dollar(Index::Unset(field))
        }, Either::<_, String>::Left)?;
        params.ensure("column");
        params.ensure("i");

        write!(&mut query,
            "DELETE FROM \"{table}\" AS \"self\"\nWHERE\n"
        ).unwrap();

        let list = self.attr.filter.infos();
        if !list.is_empty() {
            let filter = params.output(&list)?;
            write!(&mut query,
                "({filter}) AND\n"
            ).unwrap();
        }

        for (field, mut cell) in fields {
            let column = self.column(field)?;
            let list = field.attr.filter.infos();
            if !list.is_empty() {
                params.put("i", cell);
                params.put("column", Right(column));
                let filter = params.output(&list)?;
                write!(&mut query,
                    "({filter}) AND\n"
                ).unwrap();
            } else {
                let arg = params.place(&mut cell)?;
                write!(&mut query,
                    "(\"{column}\" = {arg}) AND\n"
                ).unwrap();
            }
        }
        if !query.ends_with(" AND\n") {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing delete filter";
            return Err(syn::Error::new(span, msg));
        }
        query.truncate(query.len() - 5);

        let args = params.state.0;
        self.print(&query, &args)?;
        let run = fun!(self, query, args);
        let check = self.check(&query, &args)?;
        Ok((check, run))
    }

}
