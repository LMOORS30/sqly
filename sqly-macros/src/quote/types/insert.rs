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
        if let Some(returning) = self.returning()? {
            if let Some(table) = &returning.table {
                let id = table.try_into()?;
                dep.art(Key::Table(id));
            }
        }
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}

impl InsertTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let mut local = Local::default();
        let returnable = self.returnable()?;
        let local = returnable.colocate(&mut local)?;
        let returns = returnable.returns(local)?;
        let done = self.query(&returns)?;

        let print = self.print(&done)?;
        let check = self.check(&done, &returns)?;
        let insert = self.insert(&done, &returns)?;
        let blanket = self.blanket()?;

        Ok(quote::quote! {
            #check
            #insert
            #blanket
            #print
        })
    }

}



impl InsertTable {

    pub fn query(&self, returns: &Returns<Self>) -> Result<Done<InsertTable>> {
        let mut params = Params::default();
        let mut build = Build::new(self)?;
        let table = self.table()?;

        let map = &mut params;
        let mut fields = self.cells(map, |field| {
            Dollar(Index::unset(field))
        }, |cell| cell)?;
        map.ensure("i");

        build.str(&format!(
            "INSERT INTO {table} AS \"self\" (\n"
        ))?;

        for (field, _) in &fields {
            build.opt(field, |build| {
                let column = self.column(field)?;
                build.str(&format!("\t\"{column}\",\n"))
            })?;
        }
        if !build.cut(&[",\n"])? {
            let msg = "incomplete query: missing insert column";
            return Err(syn::Error::new(Span::call_site(), msg));
        }
        build.str("\n) VALUES (\n")?;

        for (field, cell) in &mut fields {
            build.opt(field, |build| {
                let list = field.attr.insert.infos();
                if !list.is_empty() {
                    map.put("i", cell.clone());
                    build.str("\t")?;
                    build.arg(map, &list, None)?;
                    build.str(",\n")
                } else {
                    build.str("\t")?;
                    build.arg(map, &[], Some(cell))?;
                    build.str(",\n")
                }
            })?;
        }
        if !build.cut(&[",\n"])? {
            let msg = "incomplete query: missing insert value";
            return Err(syn::Error::new(Span::call_site(), msg));
        }
        build.str("\n)")?;

        build.duo(|target| returns.returns(target))?;

        let args = params.take();
        params.drain(&mut fields)?;
        let rest = params.state;
        build.done(args, rest)
    }

}
