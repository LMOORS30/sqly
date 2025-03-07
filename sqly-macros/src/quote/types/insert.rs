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

impl InsertTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let (query, args) = self.query()?;

        self.print(&query, &args)?;
        let check = self.check(&query, &args)?;
        let insert = self.insert(&query, &args, None)?;
        let blanket = self.blanket()?;

        Ok(quote::quote! {
            #check
            #insert
            #blanket
        })
    }

}



impl InsertTable {

    pub fn query(&self) -> Result<(String, Vec<&InsertField>)> {
        let mut params = Params::default();
        let mut string = String::new();
        let table = self.table()?;

        let query = &mut string;
        let fields = self.cells(&mut params, |field| {
            Dollar(Index::unset(field))
        }, |cell| cell)?;
        params.ensure("i");

        write!(query,
            "INSERT INTO \"{table}\" AS \"self\" (\n"
        ).unwrap();

        for (field, _) in &fields {
            let column = self.column(field)?;
            write!(query, "\t\"{column}\",\n").unwrap();
        }
        query.truncate(query.len() - 2);
        query.push_str("\n) VALUES (\n");

        for (field, mut cell) in fields {
            let list = field.attr.insert.infos();
            if !list.is_empty() {
                params.put("i", cell);
                query.push_str("\t");
                params.output(query, &list)?;
                query.push_str(",\n");
            } else {
                query.push_str("\t");
                params.place(query, &mut cell)?;
                query.push_str(",\n");
            }
        }
        query.truncate(query.len() - 2);
        query.push_str("\n)");

        let args = params.state;
        Ok((string, args))
    }

}
