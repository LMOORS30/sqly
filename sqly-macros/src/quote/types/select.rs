use super::*;



impl TryFrom<SelectTable> for TokenStream {
    type Error = syn::Error;

    fn try_from(table: SelectTable) -> Result<TokenStream> {
        cache::store().select(table)
    }
}

impl Cache for SelectTable {

    fn id(&self) -> Result<Id> {
        Id::try_from(&self.ident)
    }

    fn dep(&self) -> Result<Dep> {
        let mut dep = Dep::new();
        let table = &self.attr.table.data.data;
        if let Paved::Path(path) = table {
            let id = path.try_into()?;
            dep.end(Key::Table(id));
        }
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}

impl SelectTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let path = match &self.attr.table.data.data {
            Paved::Path(path) => path,
            _ => {
                let span = self.attr.table.data.span;
                let msg = "invalid table identifier: expected path\n\
                    note: #[derive(Select)] must reference a struct with #[derive(Table)]";
                return Err(syn::Error::new(span, msg));
            }
        };
        let table = cache::fetch().table(&path.try_into()?)?.sync()?;
        if spany!(table.attr.select, table.attr.from_row).is_none() {
            let span = self.attr.table.data.span;
            let msg = "missing attribute: the referenced table must have #[sqly(from_row)]\n\
                note: #[derive(Select)] uses the sqlx::FromRow definition for its query";
            return Err(syn::Error::new(span, msg));
        };

        let mut local = Local::default();
        let local = table.colocate(&mut local)?;
        let construct = table.construct(local)?;

        let query = construct.query(Target::Function, Scope::Global)?;
        let done = self.query(&query, construct.unique()?)?.map(path);
        let filter = done.query.strip_prefix(&query).unwrap_or("");

        self.print(&done.query, &done.args)?;
        let select = self.select(&done)?;
        let blanket = self.blanket()?;

        let check = self.checking(&done.args, |args| {
            let mut query = construct.query(Target::Macro, Scope::Global)?;
            query.push_str(filter);
            let krate = self.krate()?;
            match &construct.table.attr.flat {
                Some(_) => Ok(quote::quote! {
                    type Flat = <#path as #krate::Flat>::Flat;
                    #krate::sqlx::query_as!(Flat, #query #(, #args)*);
                }),
                None => {
                    let flats = construct.flats()?;
                    Ok(quote::quote! {
                        struct Flat { #(#flats,)* }
                        #krate::sqlx::query_as!(Flat, #query #(, #args)*);
                    })
                }
            }
        })?;

        Ok(quote::quote! {
            #check
            #select
            #blanket
        })
    }

}



impl<'c> Flattened<'c> {

    pub fn selects(&self) -> Result<Params<&'static str, String>> {
        let mut params = Params::default();
        params.put("table", self.construct.unique()?.to_string());
        params.put("column", self.column.column()?);
        Ok(params)
    }

    pub fn foreigns(&self) -> Result<Params<String, &'c str>> {
        let mut params = self.construct.params()?;
        let inner = match &self.nullable {
            Some(_) => ("left", "LEFT"),
            None => ("inner", "INNER"),
        };
        params.displace("left", "left");
        params.displace("LEFT", "LEFT");
        params.displace("inner", inner.0);
        params.displace("INNER", inner.1);
        if let Code::Foreign(foreign) = &self.column.code {
            params.displace("other", foreign.unique()?);
        }
        Ok(params)
    }

}



impl Construct<'_> {

    pub fn query(&self, target: Target, scope: Scope) -> Result<String> {
        let table = &self.table.attr.table.data.data;
        let mut query = String::new();
        let mut joins = String::new();
        let unique = self.unique()?;

        write!(&mut query,
            "SELECT\n"
        ).unwrap();

        for flattened in self.flatten()? {
            let flattened = flattened?;
            match (&flattened.column.code, flattened.level, scope) {
                (_, 1.., Scope::Local) => {}
                (Code::Skip, _, _) => {}
                (Code::Query, _, _) => {
                    let alias = flattened.column.alias()?;
                    let alias = match (target, scope) {
                        (Target::Function, _) => alias.to_string(),
                        (Target::Macro, Scope::Global) => format!("{alias}!: _"),
                        (Target::Macro, Scope::Local) => {
                            let modifier = flattened.column.modifier()?;
                            format!("{alias}{modifier}")
                        }
                    };
                    let list = flattened.column.field.attr.select.infos();
                    if !list.is_empty() {
                        query.push_str("\t");
                        let mut params = flattened.selects()?;
                        params.output(&mut query, &list)?;
                        write!(&mut query,
                            " AS \"{alias}\",\n"
                        ).unwrap();
                    } else {
                        let column = flattened.column.column()?;
                        let table = flattened.construct.unique()?;
                        write!(&mut query,
                            "\t\"{table}\".\"{column}\" AS \"{alias}\",\n"
                        ).unwrap();
                    }
                }
                (Code::Foreign(construct), _, Scope::Global) => {
                    let list = flattened.column.field.attr.foreign.infos();
                    if !list.is_empty() {
                        joins.push_str("\n");
                        let mut params = flattened.foreigns()?;
                        params.output(&mut joins, &list)?;
                    } else {
                        let unique = construct.unique()?;
                        let table = flattened.construct.unique()?;
                        let other = &construct.table.attr.table.data.data;
                        let resolved = construct.correlate(flattened.column)?;
                        let column = flattened.column.column()?;
                        let nullable = construct.nullable()?;
                        let foreign = resolved.column()?;
                        let join = match nullable.or(flattened.nullable) {
                            Some(_) => "LEFT JOIN",
                            None => "INNER JOIN",
                        };
                        write!(&mut joins,
                            "\n{} \"{}\" AS \"{}\" ON \"{}\".\"{}\" = \"{}\".\"{}\"",
                            join, other, unique, unique, foreign, table, column,
                        ).unwrap();
                    }
                }
                _ => {}
            }
        }
        if query.ends_with(",\n") {
            query.truncate(query.len() - 2);
        }

        write!(&mut query,
            "\nFROM \"{table}\" AS \"{unique}\""
        ).unwrap();

        query.push_str(&joins);
        Ok(query)
    }

}



impl SelectTable {

    pub fn query(&self, query: &str, table: &str) -> Result<Done<SelectTable>> {
        let mut params = Params::default();
        let mut build = Build::new(self)?;

        let map = &mut params;
        let mut fields = self.cells(map, |field| {
            Dollar(Index::unset(field))
        }, Either::<_, String>::Left)?;
        map.ensure("column");
        map.ensure("i");

        build.str(query)?;
        build.str("\nWHERE\n")?;

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
                    build.str(&format!("\t(\"{table}\".\"{column}\" = "))?;
                    build.arg(map, &[], Some(cell))?;
                    build.str(") AND\n")
                }
            })?;
        }
        if !build.cut(&[" AND\n", "\nWHERE\n"])? {
            let span = Span::call_site();
            let msg = "incomplete query: missing select filter";
            return Err(syn::Error::new(span, msg));
        }

        let args = params.take();
        params.drain(&mut fields)?;
        let rest = params.state.0;
        build.done(args.0, rest)
    }

}
