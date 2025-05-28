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
                let msg = "invalid table identifier: expected path\n\
                    note: #[derive(Select)] requires a struct with #[derive(Table)]";
                return Err(syn::Error::new(self.attr.table.data.span(), msg));
            }
        };
        let table = cache::fetch().table(&path.try_into()?)?.sync()?;
        if !table.formable() {
            let msg = "missing attribute: the referenced table must have #[sqly(from_row)]\n\
                note: #[derive(Select)] uses the sqlx::FromRow definition for its query";
            return Err(syn::Error::new_spanned(path, msg));
        };

        let mut local = Local::default();
        let local = table.colocate(&mut local)?;
        let construct = table.construct(local)?;
        let returns = Returns::Construct(path, &construct);
        let done = self.query(&construct)?;

        let print = self.print(&done)?;
        let check = self.check(&done, &returns)?;
        let select = self.select(&done, &returns)?;
        let blanket = self.blanket()?;

        Ok(quote::quote! {
            #check
            #select
            #blanket
            #print
        })
    }

}



fn selector(
    query: &mut String,
    list: &[&Info<String>],
    table: &str,
    column: &str,
    alias: &str,
    modifier: &str,
    target: Target
) -> Result<()> {
    let modifier = match target {
        Target::Macro => modifier,
        Target::Function => "",
    };
    if !list.is_empty() {
        query.push_str("\t");
        let mut params = Params::<&str, &str>::default();
        params.put("table", table);
        params.put("column", column);
        params.output(query, list)?;
        write!(query,
            " AS \"{alias}{modifier}\",\n"
        ).unwrap();
    } else if column.eq(alias) && modifier.is_empty() {
        write!(query,
            "\t\"{table}\".\"{column}\",\n"
        ).unwrap();
    } else {
        write!(query,
            "\t\"{table}\".\"{column}\" AS \"{alias}{modifier}\",\n"
        ).unwrap();
    }
    Ok(())
}

impl<T: Struct + Declare> Returns<'_, T> {

    pub fn returns(&self, target: Target) -> Result<String> {
        let mut query = String::new();

        let returns = match self {
            Returns::None => return Ok(query),
            Returns::Scalar(item) => Left(std::slice::from_ref(item)),
            Returns::Tuple(list) => Left(list.as_slice()),
            Returns::Table(_, table) => Right(table),
            Returns::Construct(_, _) => {
                let msg = "invalid query: cannot return foreign table";
                return Err(syn::Error::new(Span::call_site(), msg));
            }
        };

        write!(&mut query,
            "\nRETURNING\n"
        ).unwrap();

        match returns {
            Right(table) => table.listed(&mut query, target)?,
            Left(slice) => {
                for item in slice {
                    let table = "self";
                    let alias = item.alias()?;
                    let (column, modifier) = item.declaration()?;
                    let list = match item {
                        Scalar::Table(_, _) => Vec::new(),
                        Scalar::Paved(_, field) => field.attr.select.infos(),
                    };
                    selector(&mut query, &list, &table, &column, &alias, modifier, target)?;
                }
                if query.ends_with(",\n") {
                    query.truncate(query.len() - 2);
                }
            }
        }

        Ok(query)
    }

}

impl QueryTable {

    pub fn query(&self, target: Target) -> Result<String> {
        let table = &self.attr.table.data.data;
        let mut query = String::new();
        write!(&mut query,
            "SELECT\n"
        ).unwrap();
        self.listed(&mut query, target)?;
        write!(&mut query,
            "\nFROM {table} AS \"self\""
        ).unwrap();
        Ok(query)
    }

    fn listed(&self, query: &mut String, target: Target) -> Result<()> {
        for column in self.coded()? {
            let column = column?;
            let field = column.field;
            if let Code::Query = column.code {
                let table = "self";
                let alias = field.ident.unraw();
                let (column, modifier) = self.declaration(field)?;
                let list = field.attr.select.infos();
                selector(query, &list, &table, &column, &alias, modifier, target)?;
            }
        }
        if query.ends_with(",\n") {
            query.truncate(query.len() - 2);
        }
        Ok(())
    }

}

impl Construct<'_> {

    pub fn query(&self, target: Target) -> Result<String> {
        let table = &self.table.attr.table.data.data;
        let mut query = String::new();
        let mut joins = String::new();
        let unique = self.unique()?;

        write!(&mut query,
            "SELECT\n"
        ).unwrap();

        for flattened in self.flatten()? {
            let flattened = flattened?;
            match &flattened.column.code {
                Code::Skip => {}
                Code::Query => {
                    let modifier = "!: _";
                    let alias = flattened.column.alias()?;
                    let column = flattened.column.column()?;
                    let table = flattened.construct.unique()?;
                    let list = flattened.column.field.attr.select.infos();
                    selector(&mut query, &list, &table, &column, &alias, modifier, target)?;
                }
                Code::Foreign(construct) => {
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
            }
        }
        if query.ends_with(",\n") {
            query.truncate(query.len() - 2);
        }

        write!(&mut query,
            "\nFROM {table} AS \"{unique}\""
        ).unwrap();

        query.push_str(&joins);
        Ok(query)
    }

}



impl SelectTable {

    pub fn query(&self, construct: &Construct) -> Result<Done<SelectTable>> {
        let mut params = Params::default();
        let mut build = Build::new(self)?;
        let table = construct.unique()?;

        let map = &mut params;
        let mut fields = self.cells(map, |field| {
            Dollar(Index::unset(field))
        }, Either::<_, Cow<str>>::Left)?;
        map.ensure("column");
        map.ensure("i");

        build.duo(|target| construct.query(target))?;
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
                    build.str(&format!("\t\"{table}\".\"{column}\" = "))?;
                    build.arg(map, &[], Some(cell))?;
                    build.str(" AND\n")
                }
            })?;
        }
        if !build.cut(&[" AND\n", "\nWHERE\n"])? {
            let msg = "incomplete query: missing select filter";
            return Err(syn::Error::new(Span::call_site(), msg));
        }

        let args = params.take();
        params.drain(&mut fields)?;
        let rest = params.state.0;
        build.done(args.0, rest)
    }

}
