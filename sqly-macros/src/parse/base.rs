use crate::cache::*;
use crate::parse::*;



impl QueryTable {

    pub fn types(&self) -> Result<impl Iterator<Item = Types>> {
        let types = [
            self.attr.delete.as_ref().map(|_| Types::Delete),
            self.attr.insert.as_ref().map(|_| Types::Insert),
            self.attr.select.as_ref().map(|_| Types::Select),
            self.attr.update.as_ref().map(|_| Types::Update),
        ].into_iter().flatten();
        Ok(types)
    }

    pub fn derives(&self, r#type: Structs) -> Result<Vec<&syn::Path>> {
        let typed = match r#type {
            Structs::Flat => &self.attr.flat_derive,
            Structs::Delete => &self.attr.delete_derive,
            Structs::Insert => &self.attr.insert_derive,
            Structs::Select => &self.attr.select_derive,
            Structs::Update => &self.attr.update_derive,
        };
        let query = match r#type {
            Structs::Flat => &[],
            _ => &*self.attr.query_derive,
        };
        let derives = [query, typed].into_iter();
        let derives = derives.flatten().flat_map(|derive| {
            derive.data.iter().map(|data| &data.data)
        }).collect();
        Ok(derives)
    }

    pub fn vis(&self, r#type: Structs) -> Result<&syn::Visibility> {
        let typed = match r#type {
            Structs::Flat => &self.attr.flat_visibility,
            Structs::Delete => &self.attr.delete_visibility,
            Structs::Insert => &self.attr.insert_visibility,
            Structs::Select => &self.attr.select_visibility,
            Structs::Update => &self.attr.update_visibility,
        };
        let query = match r#type {
            Structs::Flat => &None,
            _ => &self.attr.query_visibility,
        };
        let vis = typed.as_ref().or(query.as_ref());
        let vis = vis.map(|vis| &vis.data.data);
        let vis = vis.unwrap_or(&self.vis);
        Ok(vis)
    }

    pub fn name(&self, r#type: Structs) -> Result<syn::Ident> {
        let typed = match r#type {
            Structs::Flat => &self.attr.flat,
            Structs::Delete => &self.attr.delete,
            Structs::Insert => &self.attr.insert,
            Structs::Select => &self.attr.select,
            Structs::Update => &self.attr.update,
        };
        let name = match typed.as_ref().and_then(|typed| typed.data.as_ref()) {
            Some(name) => name.data.clone(),
            None => match r#type {
                Structs::Flat => quote::format_ident!("Flat{}", self.ident),
                Structs::Delete => quote::format_ident!("Delete{}", self.ident),
                Structs::Insert => quote::format_ident!("Insert{}", self.ident),
                Structs::Select => quote::format_ident!("Select{}", self.ident),
                Structs::Update => quote::format_ident!("Update{}", self.ident),
            }
        };
        Ok(name)
    }

}



impl QueryTable {

    pub fn fielded(&self, field: &QueryField, r#type: Types) -> bool {
        !self.skipped(field, r#type.into()) && match r#type {
            Types::Delete => self.keyed(field, r#type),
            Types::Select => self.keyed(field, r#type),
            Types::Insert => true,
            Types::Update => true,
        }
    }

    pub fn skipped(&self, field: &QueryField, r#type: Skips) -> bool {
        match &field.attr.skip {
            None => false,
            Some(arr) => {
                arr.data.is_empty() ||
                arr.data.iter().any(|info| {
                    r#type == info.data
                })
            }
        }
    }

    pub fn keyed(&self, field: &QueryField, r#type: Types) -> bool {
        match &field.attr.key {
            None => false,
            Some(arr) => {
                arr.data.is_empty() ||
                arr.data.iter().any(|info| {
                    r#type == info.data.into()
                })
            }
        }
    }

}



impl QueryTable {

    pub fn selects<'c>(&'c self, field: &'c QueryField) -> Result<Vec<&'c Info<String>>> {
        let iter = field.attr.select.iter();
        let foreigns = iter.flat_map(|select| {
            &select.data
        }).collect();
        Ok(foreigns)
    }

    pub fn foreigns<'c>(&'c self, field: &'c QueryField) -> Result<Vec<&'c Info<String>>> {
        let iter = field.attr.foreign.iter();
        let foreigns = iter.flat_map(|foreign| {
            &foreign.data
        }).collect();
        Ok(foreigns)
    }

}



impl QueryTable {

    pub fn ty<'c>(&'c self, field: &'c QueryField) -> Result<&'c syn::Type> {
        let ty = match &field.attr.from {
            Some(ty) => &ty.data.data,
            None => &field.ty,
        };
        Ok(ty)
    }

}



impl Constructed<'_> {

    pub fn nullable(&self) -> Result<Option<Nullable<'_>>> {
        self.table.nullable(self.field)
    }

}



impl Construct<'_> {

    pub fn nullable(&self) -> Result<Option<Nullable<'_>>> {
        let nullable = match &self.foreign {
            Some(foreign) => foreign.nullable,
            None => None,
        };
        Ok(nullable)
    }

}

impl Resolved<'_> {

    pub fn column(&self) -> Result<String> {
        let column = match self {
            Resolved::Attr(attr) => attr.column.to_string(),
            Resolved::Field(field) => field.column()?,
        };
        Ok(column)
    }

}



impl Construct<'_> {

    pub fn unique(&self) -> Result<&str> {
        match self.unique.get() {
            Some(unique) => Ok(unique.as_str()),
            None => {
                let span = Span::call_site();
                let msg = "OnceCell not initialized\n\
                    note: this error should not occur";
                Err(syn::Error::new(span, msg))
            }
        }
    }

}

impl Constructed<'_> {

    pub fn unique(&self) -> Result<&str> {
        match self.unique.get() {
            Some(unique) => Ok(unique.as_str()),
            None => {
                let span = Span::call_site();
                let msg = "OnceCell not initialized\n\
                    note: this error should not occur";
                Err(syn::Error::new(span, msg))
            }
        }
    }

}



impl Constructed<'_> {

    pub fn column(&self) -> Result<String> {
        let table = &self.table;
        let field = &self.field;
        let named = &self.renamed()?;
        Ok(table.declaration(field, named)?.0)
    }

    pub fn modifier(&self) -> Result<String> {
        let table = &self.table;
        let field = &self.field;
        let named = &self.field.ident;
        Ok(table.declaration(field, named)?.1)
    }

    pub fn segment(&self) -> Result<String> {
        let ident = &self.field.ident;
        Ok(ident.unraw())
    }

    pub fn ident(&self) -> Result<syn::Ident> {
        let alias = self.alias()?;
        let mut ident = quote::format_ident!("r#{}", alias);
        ident.set_span(self.field.ident.span());
        Ok(ident)
    }

    pub fn alias(&self) -> Result<&str> {
        self.unique()
    }

}



macro_rules! base {
($table:ty, $field:ty) => {
both!($table, $field);

impl $table {

    pub fn table(&self) -> Result<String> {
        let guard = cache::fetch();
        let table = match &self.attr.table.data.data {
            Paved::String(table) => return Ok(table.clone()),
            Paved::Path(path) => path,
        };
        let table = guard.table(&table.try_into()?)?;
        let table = table.attr.table.data.data.clone();
        Ok(table)
    }

    pub fn fields(&self) -> Result<impl Iterator<Item = &$field>> {
        let fields = self.fields.iter().filter(|field| {
            field.attr.skip.is_none()
        });
        Ok(fields)
    }

    pub fn cells<'c, K, V, F, U, G>(
        &'c self,
        params: &mut Params<K, V>,
        mut val: F,
        mut wrap: G,
    ) -> Result<Vec<(&'c $field, V)>>
    where
        K: From<String> + Hash + Eq,
        V: Placer + Clone,
        F: FnMut(&'c $field) -> U,
        G: FnMut(Rc<RefCell<U>>) -> V,
    {
        let fields = self.fields.iter().filter_map(|field| {
            let cell = wrap(Rc::new(RefCell::new(val(field))));
            let key = field.ident.unraw();
            match &field.attr.skip {
                None => {
                    params.put(key, cell.clone());
                    Some((field, cell))
                }
                Some(_) => {
                    params.put(key, cell);
                    None
                }
            }
        }).collect();
        Ok(fields)
    }

}

} }



macro_rules! both {
($table:ty, $field:ty) => {

impl $table {

    pub fn rename(&self, field: &$field, string: &str) -> Result<String> {
        let all = &self.attr.rename;
        let rename = &field.attr.rename;
        let renamed = match rename.as_ref().or(all.as_ref()) {
            Some(re) => re.data.data.rename(string),
            None => string.to_string(),
        };
        Ok(renamed)
    }

    pub fn declaration(&self, field: &$field, named: &syn::Ident) -> Result<(String, String)> {
        const SEP: &'static [char] = &['!', '?', ':'];

        let iden;
        let name = match &field.attr.column {
            Some(column) => &column.data.data,
            None => {
                iden = named.to_string();
                match iden.strip_prefix("r#") {
                    Some(strip) => strip,
                    None => &iden,
                }
            }
        };

        let (name, info) = match name.find(SEP) {
            Some(i) => name.split_at(i),
            None => (name, ""),
        };

        let name = self.rename(field, name)?;
        let info = match (info.chars().next(), &field.attr.infer) {
            (Some('!'), Some(_)) => "!: _".to_string(),
            (Some('?'), Some(_)) => "?: _".to_string(),
            (_, Some(_)) => ": _".to_string(),
            (_, None) => info.to_string(),
        };

        Ok((name, info))
    }

    pub fn column(&self, field: &$field) -> Result<String> {
        Ok(self.declaration(field, &field.ident)?.0)
    }
    
    pub fn modifier(&self, field: &$field) -> Result<String> {
        Ok(self.declaration(field, &field.ident)?.1)
    }

}

impl $table {

    #[cfg(feature = "unchecked")]
    pub fn checked(&self) -> bool {
        false
    }

    #[cfg(not(feature = "unchecked"))]
    pub fn checked(&self) -> bool {
        self.attr.unchecked.is_none()
    }

}

} }



both!(QueryTable, QueryField);
base!(DeleteTable, DeleteField);
base!(InsertTable, InsertField);
base!(SelectTable, SelectField);
base!(UpdateTable, UpdateField);
