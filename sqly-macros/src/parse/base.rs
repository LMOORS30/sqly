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
            !field.skipped(r#type) && match r#type {
                crate::parse::Types::Delete => field.keyed(r#type),
                crate::parse::Types::Select => field.keyed(r#type),
                crate::parse::Types::Insert => true,
                crate::parse::Types::Update => true,
            }
        });
        Ok(fields)
    }

}



impl QueryField {

    pub fn skipped(&self, r#type: Types) -> bool {
        match &self.attr.skip {
            None => false,
            Some(arr) => {
                arr.data.is_empty() ||
                arr.data.iter().any(|info| {
                    r#type == info.data.into()
                })
            },
        }
    }

    pub fn keyed(&self, r#type: Types) -> bool {
        match &self.attr.key {
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



macro_rules! both {
($table:ty, $field:ty) => {
base!($table, $field);

impl $table {

    pub fn fields(&self) -> Result<impl Iterator<Item = &$field>> {
        let fields = self.fields.iter().filter(|field| {
            field.attr.skip.is_none()
        });
        Ok(fields)
    }

}

} }



macro_rules! base {
($table:ty, $field:ty) => {

impl $table {

    pub fn column(&self, field: &$field) -> Result<String> {
        let name = match &field.attr.column {
            Some(column) => column.data.data.clone(),
            None => field.ident.to_string(),
        };
        let all = &self.attr.rename;
        let rename = &field.attr.rename;
        let name = match rename.as_ref().or(all.as_ref()) {
            Some(re) => re.data.data.rename(&name),
            None => name,
        };
        Ok(name)
    }

}

} }



base!(QueryTable, QueryField);
both!(DeleteTable, DeleteField);
both!(InsertTable, InsertField);
both!(SelectTable, SelectField);
both!(UpdateTable, UpdateField);