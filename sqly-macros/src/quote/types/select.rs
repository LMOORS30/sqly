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



impl<'c> Construct<'c> {

    pub fn query(&self, target: Target) -> Result<String> {
        let table = &self.table.attr.table.data.data;
        let mut query = String::new();
        let mut joins = String::new();
        let unique = self.unique()?;

        write!(&mut query,
            "SELECT\n"
        ).unwrap();

        let mut i = 1;
        for flattened in self.flattened()? {
            let flattened = flattened?;
            match &flattened.column.code {
                Code::Skip => {},
                Code::Query => {
                    let alias = flattened.column.alias()?;
                    let alias = match target {
                        Target::Function => alias.to_string(),
                        Target::Macro => format!("{alias}!: _"),
                    };
                    let column = flattened.column;
                    let list = column.table.selects(column.field)?;
                    if !list.is_empty() {
                        query.push_str("\t");
                        let params = flattened.selects(&alias)?;
                        let select = list.into_iter().map(|select| {
                            params.replace(&select.data, select.span)
                        }).collect::<Result<String>>()?;
                        query.push_str(&select);
                        query.push_str(",\n");
                    }
                    else {
                        let column = flattened.column.column()?;
                        let table = flattened.construct.unique()?;
                        write!(&mut query,
                            "\t\"{table}\".\"{column}\" AS \"{alias}\",\n"
                        ).unwrap();
                    }
                    i += 1;
                },
                Code::Foreign(construct) => {
                    let column = flattened.column;
                    let list = column.table.foreigns(column.field)?;
                    if !list.is_empty() {
                        let params = flattened.foreigns(construct)?;
                        let foreign = list.into_iter().map(|foreign| {
                            params.replace(&foreign.data, foreign.span)
                        }).collect::<Result<String>>()?;
                        joins.push_str(&foreign);
                    }
                    else {
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

        let mut query = construct.query(Target::Function)?;
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
            let mut query = construct.query(Target::Macro)?;
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
