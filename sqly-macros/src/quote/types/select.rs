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
        let (check, query) = self.query()?;
        let typle = self.typle()?;
        let ident = &self.ident;
        let res = result!['q, typle];

        Ok(quote::quote! {
            #check
            #[automatically_derived]
            impl ::sqly::Select for #ident {
                type Table = #typle;
                type Query<'q> = #res;
                fn select(&self) -> Self::Query<'_> {
                    #query
                }
            }
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

        let mut i = 1;
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
                    i += 1;
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
        let trunc = if i > 1 { 2 } else { 1 };
        query.truncate(query.len() - trunc);

        write!(&mut query,
            "\nFROM \"{table}\" AS \"{unique}\""
        ).unwrap();

        query.push_str(&joins);
        Ok(query)
    }

}



impl SelectTable {

    pub fn query(&self) -> Result<(TokenStream, TokenStream)> {
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

        let mut local = Local::default();
        let local = table.colocate(&mut local)?;
        let construct = table.construct(local)?;
        let table = construct.unique()?;

        let mut query = construct.query(Target::Function, Scope::Global)?;
        let params = &mut Params::default();
        let select = &mut String::new();

        let fields = self.cells(params, |field| {
            Dollar(Index::unset(field))
        }, Either::<_, String>::Left)?;
        params.ensure("column");
        params.ensure("i");

        write!(select,
            "\nWHERE\n"
        ).unwrap();

        let list = self.attr.filter.infos();
        if !list.is_empty() {
            select.push_str("\t(");
            params.output(select, &list)?;
            select.push_str(") AND\n");
        }

        for (field, mut cell) in fields {
            let column = self.column(field)?;
            let list = field.attr.filter.infos();
            if !list.is_empty() {
                params.put("i", cell);
                params.put("column", Right(column));
                select.push_str("\t(");
                params.output(select, &list)?;
                select.push_str(") AND\n");
            } else {
                write!(select, "\t(\"{table}\".\"{column}\" = ").unwrap();
                params.place(select, &mut cell)?;
                select.push_str(") AND\n");
            }
        }
        if !select.ends_with(" AND\n") {
            select.truncate(select.len() - 2);
        }
        select.truncate(select.len() - 5);
        query.push_str(&select);

        let args = &params.state.0;
        self.print(&query, args)?;
        let run = fun!(self, query, args);

        let check = self.checking(args, |args| {
            let mut query = construct.query(Target::Macro, Scope::Global)?;
            query.push_str(&select);
            match &construct.table.attr.flat {
                Some(_) => Ok(quote::quote! {
                    type Flat = <#path as ::sqly::Flat>::Flat;
                    ::sqlx::query_as!(Flat, #query #(, #args)*);
                }),
                None => {
                    let flats = construct.flats()?;
                    Ok(quote::quote! {
                        struct Flat { #(#flats,)* }
                        ::sqlx::query_as!(Flat, #query #(, #args)*);
                    })
                }
            }
        })?;

        let run = quote::quote! {
            #run.try_map(<#path as ::sqly::Table>::from_row)
        };

        Ok((check, run))
    }

}
