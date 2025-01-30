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
        let (check, query) = self.query()?;
        let ident = &self.ident;
        let res = result!['q];

        Ok(quote::quote! {
            #check
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

    pub fn query(&self) -> Result<(TokenStream, TokenStream)> {
        let mut params = Params::default();
        let mut query = String::new();
        let table = self.table()?;

        let fields = self.cells(&mut params, |field| {
            Dollar(Index::Unset(field))
        }, |cell| cell)?;
        params.ensure("i");

        write!(&mut query,
            "INSERT INTO \"{table}\" AS \"self\" (\n"
        ).unwrap();

        for field in self.fields()? {
            let column = self.column(field)?;
            write!(&mut query,
                "\t\"{column}\",\n"
            ).unwrap();
        }
        query.truncate(query.len() - 2);
        query.push_str("\n) VALUES (\n");

        for (field, mut cell) in fields {
            let list = field.attr.insert.infos();
            if !list.is_empty() {
                params.put("i", cell);
                let arg = params.output(&list)?;
                write!(&mut query,
                    "\t{arg},\n"
                ).unwrap();
            } else {
                let arg = params.place(&mut cell)?;
                write!(&mut query,
                    "\t{arg},\n"
                ).unwrap();
            }
        }
        query.truncate(query.len() - 2);
        query.push_str("\n)");

        let args = params.state;
        self.print(&query, &args)?;
        let run = fun!(self, query, args);
        let check = self.check(&query, &args)?;
        Ok((check, run))
    }

}
