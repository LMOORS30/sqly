use syn::Result;

use super::*;



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

    pub fn derives(&self, r#type: Types) -> Result<Vec<syn::Path>> {
        let typed = match r#type {
            Types::Delete => &self.attr.delete_derive,
            Types::Insert => &self.attr.insert_derive,
            Types::Select => &self.attr.select_derive,
            Types::Update => &self.attr.update_derive,
        };
        let query = &self.attr.query_derive;
        let derives = [query, typed].into_iter();
        let derives = derives.flatten().flat_map(|derive| {
            derive.data.iter().map(|data| data.data.clone())
        }).collect();
        Ok(derives)
    }

    pub fn vis(&self, r#type: Types) -> Result<syn::Visibility> {
        let typed = match r#type {
            Types::Delete => &self.attr.delete_visibility,
            Types::Insert => &self.attr.insert_visibility,
            Types::Select => &self.attr.select_visibility,
            Types::Update => &self.attr.update_visibility,
        };
        let query = &self.attr.query_visibility;
        let vis = typed.as_ref().or(query.as_ref());
        let vis = vis.map(|vis| vis.data.data.clone());
        let vis = vis.unwrap_or_else(|| self.vis.clone());
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

    pub fn fields(&self, r#type: Types) -> Result<impl Iterator<Item = &QueryField>> {
        let fields = self.fields.iter().filter(move |field| {
            !self.skipped(field, r#type.into()) && match r#type {
                crate::parse::Types::Delete => self.keyed(field, r#type),
                crate::parse::Types::Select => self.keyed(field, r#type),
                crate::parse::Types::Insert => true,
                crate::parse::Types::Update => true,
            }
        });
        Ok(fields)
    }

    pub fn skipped(&self, field: &QueryField, r#type: Skips) -> bool {
        match &field.attr.skip {
            None => false,
            Some(arr) => {
                arr.data.is_empty() ||
                arr.data.iter().any(|info| {
                    r#type == info.data
                })
            },
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
            },
        }
    }

}



impl QueryTable {

    pub fn columns(&self) -> Result<impl Iterator<Item = &QueryField>> {
        let columns = self.fields.iter().filter(|field| {
            !self.skipped(field, Skips::Query)
        });
        Ok(columns)
    }

}



macro_rules! base {
($table:ty, $field:ty) => {
both!($table, $field);

impl $table {

    pub fn table(&self) -> Result<String> {
        let guard = crate::cache::fetch();
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

} }



macro_rules! both {
($table:ty, $field:ty) => {

impl $table {
    
}

} }



both!(QueryTable, QueryField);
base!(DeleteTable, DeleteField);
base!(InsertTable, InsertField);
base!(SelectTable, SelectField);
base!(UpdateTable, UpdateField);
