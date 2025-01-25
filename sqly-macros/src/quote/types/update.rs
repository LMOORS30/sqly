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
        })?;
        params.ensure("i");

        write!(&mut query,
            "UPDATE \"{table}\" SET\n",
        ).unwrap();

        let mut i = 1;
        for (field, cell) in &mut fields {
            if field.attr.key.is_none() {
                params.put("i", cell.clone());
                let column = self.column(field)?;
                let list = field.attr.update.infos();
                let arg = match list.is_empty() {
                    false => params.output(&list)?,
                    true => params.place(cell)?,
                };
                write!(&mut query,
                    "\t\"{column}\" = {arg},\n"
                ).unwrap();
                i += 1;
            }
        }
        let trunc = if i > 1 { 2 } else { 5 };
        query.truncate(query.len() - trunc);
        query.push_str("\nWHERE\n");

        let mut j = i;
        for (field, cell) in &mut fields {
            if field.attr.key.is_some() {
                let column = self.column(field)?;
                let arg = params.place(cell)?;
                write!(&mut query,
                    "\t\"{column}\" = {arg} AND\n"
                ).unwrap();
                j += 1;
            }
        }
        let trunc = if j > i { 5 } else { 7 };
        query.truncate(query.len() - trunc);

        let args = params.state;
        self.print(&query, &args)?;
        let run = fun!(self, query, args);
        let check = self.check(&query, &args)?;
        Ok(quote::quote! { #check #run })
    }

}
