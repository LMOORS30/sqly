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
        if let Paved::Path(path) = table {
            dep.art(Key::Table(path));
        }
        if let Some(returning) = self.returning()? {
            if let Some(path) = &returning.table {
                dep.art(Key::Table(path));
            }
        }
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}

impl DeleteTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let mut local = Local::default();
        let returnable = self.returnable()?;
        let local = returnable.colocate(&mut local)?;
        let returns = returnable.returns(local)?;
        let done = self.query(&returns)?;

        let print = self.print(&done)?;
        let check = self.check(&done, &returns)?;
        let delete = self.delete(&done, &returns)?;
        let blanket = self.blanket()?;

        Ok(quote::quote! {
            #check
            #delete
            #blanket
            #print
        })
    }

}



impl DeleteTable {

    pub fn query(&self, returns: &Returns<Self>) -> Result<Done<DeleteTable>> {
        let mut params = Params::default();
        let mut build = Build::new(self)?;
        let table = self.table()?;
        let unique = "self";

        let map = &mut params;
        let mut fields = self.cells(map, |field| {
            Dollar(Index::unset(field))
        }, Either::<_, Cow<_>>::Left)?;
        map.ensure("column");
        map.ensure("i");

        build.str(&format!(
            "DELETE FROM {table} AS \"{unique}\"\nWHERE\n"
        ))?;

        map.displace("table", Right(unique.into()));
        let list = self.attr.filter.infos();
        if !list.is_empty() {
            build.str("\t(")?;
            build.arg(map, &list, None)?;
            build.str(") AND\n")?;
        }

        for (field, cell) in &mut fields {
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
                    build.str(&format!("\t\"{column}\" = "))?;
                    build.arg(map, &[], Some(cell))?;
                    build.str(" AND\n")
                }
            })?;
        }
        if !build.cut(&[" AND\n", "\nWHERE\n"])? {
            let msg = "incomplete query: missing delete filter";
            return Err(syn::Error::new(Span::call_site(), msg));
        }

        build.duo(|target| returns.returns(target))?;

        let args = params.take();
        params.drain(&mut fields)?;
        let rest = params.state.0;
        build.done(args.0, rest)
    }

}
