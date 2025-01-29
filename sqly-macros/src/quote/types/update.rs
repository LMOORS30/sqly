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
                    let obj = self;
                    #query
                }
            }
        })
    }

}



impl UpdateTable {

    pub fn query(&self) -> Result<TokenStream> {
        let mut params = Params::default();
        let mut query = String::new();
        let table = self.table()?;

        let mut fields = self.cells(&mut params, |field| {
            Dollar(Index::Unset(field))
        }, Either::<_, String>::Left)?;
        params.ensure("column");
        params.ensure("i");

        write!(&mut query,
            "UPDATE \"{table}\" AS \"self\"\nSET\n",
        ).unwrap();

        for (field, cell) in &mut fields {
            if field.attr.key.is_none() {
                let column = self.column(field)?;
                let list = field.attr.update.infos();
                write!(&mut query,
                    "\t\"{column}\" = "
                ).unwrap();
                if !list.is_empty() {
                    params.put("i", cell.clone());
                    params.put("column", Right(column));
                    let arg = params.output(&list)?;
                    write!(&mut query,
                        "{arg},\n"
                    ).unwrap();
                } else {
                    let arg = params.place(cell)?;
                    write!(&mut query,
                        "{arg},\n"
                    ).unwrap();
                }
            }
        }
        query.truncate(query.len() - 2);
        query.push_str("\nWHERE\n");

        let list = self.attr.filter.infos();
        if !list.is_empty() {
            let filter = params.output(&list)?;
            write!(&mut query,
                "({filter}) AND\n"
            ).unwrap();
        }

        for (field, mut cell) in fields {
            if field.attr.key.is_some() {
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
        }
        if !query.ends_with(" AND\n") {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing update filter";
            return Err(syn::Error::new(span, msg));
        }
        query.truncate(query.len() - 5);

        let args = params.state.0;
        self.print(&query, &args)?;
        let run = fun!(self, query, args);
        let check = self.check(&query, &args)?;
        Ok(quote::quote! { #check #run })
    }

}
