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
        if let Paved::Path(path) = table {
            let id = path.try_into()?;
            dep.art(Key::Table(id));
        }
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}

impl UpdateTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let (query, args) = self.query()?;

        self.print(&query, &args)?;
        let check = self.check(&query, &args)?;
        let update = self.update(&query, &args, None)?;
        let blanket = self.blanket()?;

        Ok(quote::quote! {
            #check
            #update
            #blanket
        })
    }

}



impl UpdateTable {

    pub fn query(&self) -> Result<(String, Vec<&UpdateField>)> {
        let mut params = Params::default();
        let mut string = String::new();
        let table = self.table()?;

        let query = &mut string;
        let mut fields = self.cells(&mut params, |field| {
            Dollar(Index::unset(field))
        }, Either::<_, String>::Left)?;
        params.ensure("column");
        params.ensure("i");

        write!(query,
            "UPDATE \"{table}\" AS \"self\"\nSET\n",
        ).unwrap();

        for (field, cell) in &mut fields {
            if field.attr.key.is_none() {
                let column = self.column(field)?;
                let list = field.attr.update.infos();
                write!(query, "\t\"{column}\" = ").unwrap();
                if !list.is_empty() {
                    params.put("i", cell.clone());
                    params.put("column", Right(column));
                    params.output(query, &list)?;
                    query.push_str(",\n");
                } else {
                    params.place(query, cell)?;
                    query.push_str(",\n");
                }
            }
        }
        query.truncate(query.len() - 2);
        query.push_str("\nWHERE\n");

        let list = self.attr.filter.infos();
        if !list.is_empty() {
            query.push_str("\t(");
            params.output(query, &list)?;
            query.push_str(") AND\n");
        }

        for (field, mut cell) in fields {
            if field.attr.key.is_some() {
                let column = self.column(field)?;
                let list = field.attr.filter.infos();
                if !list.is_empty() {
                    params.put("i", cell);
                    params.put("column", Right(column));
                    query.push_str("\t(");
                    params.output(query, &list)?;
                    query.push_str(") AND\n");
                } else {
                    write!(query, "\t(\"{column}\" = ").unwrap();
                    params.place(query, &mut cell)?;
                    query.push_str(") AND\n");
                }
            }
        }
        if !query.ends_with(" AND\n") {
            let span = Span::call_site();
            let msg = "incomplete query: missing update filter";
            return Err(syn::Error::new(span, msg));
        }
        query.truncate(query.len() - 5);

        let args = params.state.0;
        Ok((string, args))
    }

}
