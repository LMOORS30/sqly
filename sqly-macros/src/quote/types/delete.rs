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
            let id = path.try_into()?;
            dep.art(Key::Table(id));
        }
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}

impl DeleteTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let done = self.query()?;

        self.print(&done.query, &done.args)?;
        let check = self.check(&done.query, &done.args)?;
        let delete = self.delete(&done)?;
        let blanket = self.blanket()?;

        Ok(quote::quote! {
            #check
            #delete
            #blanket
        })
    }

}



impl DeleteTable {

    pub fn query(&self) -> Result<Done<DeleteTable>> {
        let mut params = Params::default();
        let mut build = Build::new(self)?;
        let table = self.table()?;

        let map = &mut params;
        let mut fields = self.cells(map, |field| {
            Dollar(Index::unset(field))
        }, Either::<_, String>::Left)?;
        map.ensure("column");
        map.ensure("i");

        build.str(&format!(
            "DELETE FROM \"{table}\" AS \"self\"\nWHERE\n"
        ))?;

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
                    build.str(&format!("\t(\"{column}\" = "))?;
                    build.arg(map, &[], Some(cell))?;
                    build.str(") AND\n")
                }
            })?;
        }
        if !build.cut(&[" AND\n"])? {
            let span = Span::call_site();
            let msg = "incomplete query: missing delete filter";
            return Err(syn::Error::new(span, msg));
        }

        let args = params.take();
        params.drain(&mut fields)?;
        let rest = params.state.0;
        build.done(args.0, rest)
    }

}
