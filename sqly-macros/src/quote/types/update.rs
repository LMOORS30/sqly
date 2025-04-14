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

impl UpdateTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let mut local = Local::default();
        let returnable = self.returnable()?;
        let local = returnable.colocate(&mut local)?;
        let returns = returnable.returns(local)?;
        let done = self.query(&returns)?;

        let print = self.print(&done)?;
        let check = self.check(&done, &returns)?;
        let update = self.update(&done, &returns)?;
        let blanket = self.blanket()?;

        Ok(quote::quote! {
            #check
            #update
            #blanket
            #print
        })
    }

}



impl UpdateTable {

    pub fn query(&self, returns: &Returns<Self>) -> Result<Done<UpdateTable>> {
        let mut params = Params::default();
        let mut build = Build::new(self)?;
        let table = self.table()?;

        let map = &mut params;
        let mut fields = self.cells(map, |field| {
            Dollar(Index::unset(field))
        }, Either::<_, Cow<str>>::Left)?;
        map.ensure("column");
        map.ensure("i");

        build.str(&format!(
            "UPDATE \"{table}\" AS \"self\"\nSET\n",
        ))?;

        for (field, cell) in &mut fields {
            if field.attr.key.is_none() {
                build.opt(field, |build| {
                    let column = self.column(field)?;
                    let list = field.attr.update.infos();
                    build.str(&format!("\t\"{column}\" = "))?;
                    if !list.is_empty() {
                        map.put("i", cell.clone());
                        map.put("column", Right(column));
                    }
                    build.arg(map, &list, Some(cell))?;
                    build.str(",\n")
                })?;
            }
        }
        if !build.cut(&[",\n"])? {
            let msg = "incomplete query: missing update column";
            return Err(syn::Error::new(Span::call_site(), msg));
        }
        build.str("\nWHERE\n")?;

        let list = self.attr.filter.infos();
        if !list.is_empty() {
            build.str("\t(")?;
            build.arg(map, &list, None)?;
            build.str(") AND\n")?;
        }

        for (field, cell) in &mut fields {
            if field.attr.key.is_some() {
                build.opt(field, |build| {
                    let column = self.column(field)?;
                    let list = field.attr.filter.infos();
                    if !list.is_empty() {
                        map.put("i", cell.clone());
                        map.put("column", Right(column));
                        build.str("\t(")?;
                        build.arg(map, &list, None)?;
                        build.str(") AND\n")
                    } else {
                        build.str(&format!("\t(\"{column}\" = "))?;
                        build.arg(map, &[], Some(cell))?;
                        build.str(") AND\n")
                    }
                })?;
            }
        }
        if !build.cut(&[" AND\n"])? {
            let msg = "incomplete query: missing update filter";
            return Err(syn::Error::new(Span::call_site(), msg));
        }

        build.duo(|target| returns.returns(target))?;

        let args = params.take();
        params.drain(&mut fields)?;
        let rest = params.state.0;
        build.done(args.0, rest)
    }

}
