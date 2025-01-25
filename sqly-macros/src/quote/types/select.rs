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
        dep.end(Key::Table(table.try_into()?));
        Ok(dep)
    }

    fn call(self) -> Result<TokenStream> {
        self.debug(self.derived()?)
    }

}

impl SelectTable {

    pub fn derived(&self) -> Result<TokenStream> {
        let table = &self.attr.table.data.data;
        let query = self.query()?;
        let ident = &self.ident;
        let res = result!['q, table];

        Ok(quote::quote! {
            #[automatically_derived]
            impl ::sqly::Select for #ident {
                type Table = #table;
                type Query<'q> = #res;
                fn select(&self) -> Self::Query<'_> {
                    let obj = self;
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
        let inner = match self.optional {
            Some(_) => ("left", "LEFT"),
            None => ("inner", "INNER"),
        };
        params.emplace("left", "left");
        params.emplace("LEFT", "LEFT");
        params.emplace("inner", inner.0);
        params.emplace("INNER", inner.1);
        if let Code::Foreign(foreign) = &self.column.code {
            params.emplace("other", foreign.unique()?);
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
                        let mut params = flattened.selects()?;
                        let select = params.output(&list)?;
                        write!(&mut query,
                            "\t{select} AS \"{alias}\",\n"
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
                        let foreign = params.output(&list)?;
                        joins.push_str(&foreign);
                    } else {
                        let unique = construct.unique()?;
                        let table = flattened.construct.unique()?;
                        let other = &construct.table.attr.table.data.data;
                        let resolved = construct.correlate(flattened.column)?;
                        let column = flattened.column.column()?;
                        let optional = construct.optional()?;
                        let foreign = resolved.column()?;
                        let join = match optional.or(flattened.optional) {
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

    pub fn query(&self) -> Result<TokenStream> {
        let res = &self.attr.table.data.data;
        let table = cache::fetch().table(&res.try_into()?)?.sync()?;

        let mut local = Local::default();
        let local = table.colocate(&mut local)?;
        let construct = table.construct(local)?;
        let table = construct.unique()?;

        let mut query = construct.query(Target::Function, Scope::Global)?;
        let mut select = String::new();
        let mut args = Vec::new();

        write!(&mut select,
            "\nWHERE\n"
        ).unwrap();

        let mut i = 1;
        for field in self.fields()? {
            let column = self.column(field)?;
            write!(&mut select,
                "\t\"{table}\".\"{column}\" = ${i} AND\n"
            ).unwrap();
            args.push(field);
            i += 1;
        }
        let trunc = if i > 1 { 5 } else { 7 };
        select.truncate(select.len() - trunc);

        query.push_str(&select);
        self.print(&query, &args)?;
        let run = fun!(self, query, args);

        let check = self.checked(&args, |args| {
            let mut query = construct.query(Target::Macro, Scope::Global)?;
            query.push_str(&select);
            Ok(quote::quote! {
                type Flat = <#res as ::sqly::Table>::Flat;
                ::sqlx::query_as!(Flat, #query #(, #args)*);
            })
        })?;

        Ok(quote::quote! {
            #check
            #run.try_map(<#res as ::sqly::Table>::from_row)
        })
    }

}
