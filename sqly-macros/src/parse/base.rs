use syn::Result;

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

    pub fn derives(&self, r#type: Types) -> Result<Vec<&syn::Path>> {
        let typed = match r#type {
            Types::Delete => &self.attr.delete_derive,
            Types::Insert => &self.attr.insert_derive,
            Types::Select => &self.attr.select_derive,
            Types::Update => &self.attr.update_derive,
        };
        let query = &self.attr.query_derive;
        let derives = [query, typed].into_iter();
        let derives = derives.flatten().flat_map(|derive| {
            derive.data.iter().map(|data| &data.data)
        }).collect();
        Ok(derives)
    }

    pub fn vis(&self, r#type: Types) -> Result<&syn::Visibility> {
        let typed = match r#type {
            Types::Delete => &self.attr.delete_visibility,
            Types::Insert => &self.attr.insert_visibility,
            Types::Select => &self.attr.select_visibility,
            Types::Update => &self.attr.update_visibility,
        };
        let query = &self.attr.query_visibility;
        let vis = typed.as_ref().or(query.as_ref());
        let vis = vis.map(|vis| &vis.data.data);
        let vis = vis.unwrap_or(&self.vis);
        Ok(vis)
    }

    pub fn name(&self, r#type: Types) -> Result<syn::Ident> {
        let typed = match r#type {
            Types::Delete => &self.attr.delete,
            Types::Insert => &self.attr.insert,
            Types::Select => &self.attr.select,
            Types::Update => &self.attr.update,
        };
        let name = match typed.as_ref().and_then(|typed| typed.data.as_ref()) {
            Some(name) => name.data.clone(),
            None => match r#type {
                Types::Delete => quote::format_ident!("Delete{}", self.ident),
                Types::Insert => quote::format_ident!("Insert{}", self.ident),
                Types::Select => quote::format_ident!("Select{}", self.ident),
                Types::Update => quote::format_ident!("Update{}", self.ident),
            }
        };
        Ok(name)
    }

    pub fn flat(&self) -> Result<syn::Ident> {
        let row = match &self.attr.flat {
            Some(row) => row.data.data.clone(),
            None => quote::format_ident!("Flat{}", self.ident),
        };
        Ok(row)
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

    pub fn optional(&self) -> Result<Option<Optional<'_>>> {
        self.table.optional(self.field)
    }

}



impl Construct<'_> {

    pub fn optional(&self) -> Result<Option<Optional<'_>>> {
        let optional = match &self.foreign {
            Some(foreign) => foreign.optional,
            None => None,
        };
        Ok(optional)
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
                let msg = "OnceCell not initialized\n\
                    note: this error should not occur";
                let span = proc_macro2::Span::call_site();
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
                let msg = "OnceCell not initialized\n\
                    note: this error should not occur";
                let span = proc_macro2::Span::call_site();
                Err(syn::Error::new(span, msg))
            }
        }
    }

}



impl Constructed<'_> {

    pub fn column(&self) -> Result<String> {
        let table = &self.table;
        let field = &self.field;
        let named = &self.named()?;
        Ok(table.declaration(field, named)?.0)
    }

    pub fn modifier(&self) -> Result<String> {
        let table = &self.table;
        let field = &self.field;
        let named = &self.field.ident;
        Ok(table.declaration(field, named)?.1)
    }

    pub fn segment(&self) -> Result<String> {
        let ident = self.field.ident.to_string();
        let unique = match ident.strip_prefix("r#") {
            Some(strip) => strip.to_string(),
            None => ident,
        };
        Ok(unique)
    }

    pub fn ident(&self) -> Result<syn::Ident> {
        Ok(quote::format_ident!("r#{}", self.alias()?))
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
        let table = &self.attr.table.data.data;
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

}

impl $table {

    pub fn column(&self, field: &$field) -> Result<String> {
        Ok(self.declaration(field, &field.ident)?.0)
    }
    
    pub fn modifier(&self, field: &$field) -> Result<String> {
        Ok(self.declaration(field, &field.ident)?.1)
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
                &iden
            }
        };

        let (name, info) = match name.find(SEP) {
            Some(i) => name.split_at(i),
            None => (name.as_str(), ""),
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

}

} }



both!(QueryTable, QueryField);
base!(DeleteTable, DeleteField);
base!(InsertTable, InsertField);
base!(SelectTable, SelectField);
base!(UpdateTable, UpdateField);
