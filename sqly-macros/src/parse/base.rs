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

    pub fn name(&self, r#type: Structs) -> Result<Cow<syn::Ident>> {
        let typed = match r#type {
            Structs::Flat => &self.attr.flat,
            Structs::Delete => &self.attr.delete,
            Structs::Insert => &self.attr.insert,
            Structs::Select => &self.attr.select,
            Structs::Update => &self.attr.update,
        };
        let name = match typed.as_ref().and_then(|typed| typed.data.as_ref()) {
            Some(name) => Cow::Borrowed(&name.data),
            None => Cow::Owned(match r#type {
                Structs::Flat => quote::format_ident!("Flat{}", self.ident),
                Structs::Delete => quote::format_ident!("Delete{}", self.ident),
                Structs::Insert => quote::format_ident!("Insert{}", self.ident),
                Structs::Select => quote::format_ident!("Select{}", self.ident),
                Structs::Update => quote::format_ident!("Update{}", self.ident),
            }),
        };
        Ok(name)
    }

}



impl QueryTable {

    pub fn optional(&self, field: &QueryField, r#type: Types) -> Option<Span> {
        let optional = match r#type {
            Types::Insert => &field.attr.insert_optional,
            Types::Update => &field.attr.update_optional,
            Types::Delete | Types::Select => &None,
        };
        if let Some(opt) = optional {
            return opt.data.as_ref().map_or(true, |opt| opt.data).then(|| opt.span);
        }
        let optional = match r#type {
            Types::Insert => &self.attr.insert_optional,
            Types::Update => &self.attr.update_optional,
            Types::Delete | Types::Select => &None,
        };
        if let Some(opt) = optional {
            return match r#type {
                Types::Update => (!self.keyed(field, r#type)).then(|| opt.span),
                Types::Delete | Types::Insert | Types::Select => Some(opt.span),
            }
        }
        None
    }

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

    pub fn ty<'c>(&'c self, field: &'c QueryField) -> Result<&'c syn::Type> {
        let ty = match (&field.attr.try_from, &field.attr.from) {
            (Some(ty), _) => &ty.data.data,
            (_, Some(ty)) => &ty.data.data,
            _ => &field.ty,
        };
        Ok(ty)
    }

}



impl Nullable<'_> {

    pub fn defaulted(&self) -> bool {
        match self {
            Nullable::Option(_) => false,
            Nullable::Default(_, _) => true,
        }
    }

}

impl Constructed<'_> {

    pub fn defaulted(&self) -> Result<Option<Nullable>> {
        self.table.defaulted(self.field)
    }

    pub fn nullable(&self) -> Result<Option<Nullable>> {
        self.table.nullable(self.field)
    }

}

impl Construct<'_> {

    pub fn nullable(&self) -> Result<Option<Nullable>> {
        let nullable = match &self.foreign {
            Some(foreign) => foreign.nullable,
            None => None,
        };
        Ok(nullable)
    }

}

impl Resolved<'_> {

    pub fn column(&self) -> Result<Cow<str>> {
        let column = match self {
            Resolved::Attr(attr) => Cow::Borrowed(attr.column),
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
                Err(syn::Error::new(Span::call_site(), msg))
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
                Err(syn::Error::new(Span::call_site(), msg))
            }
        }
    }

}



impl Declared for Constructed<'_> {

    fn declaration(&self) -> Result<Declaration> {
        let named = self.renamed()?;
        self.table.declare(self.field, &named)
    }

}

impl Constructed<'_> {

    pub fn ident(&self) -> Result<Cow<syn::Ident>> {
        let alias = self.alias()?;
        if alias.eq(&self.field.ident.unraw()) {
            return Ok(Cow::Borrowed(&self.field.ident));
        }
        let span = self.field.ident.span();
        let ident = syn::Ident::new(alias, span);
        Ok(Cow::Owned(ident))
    }

    pub fn alias(&self) -> Result<&str> {
        self.unique()
    }

}

impl<T: Struct + Declare> Declared for Scalar<'_, T> {

    fn declaration(&self) -> Result<Declaration> {
        let declaration = match self {
            Scalar::Table(table, field) => table.declaration(field)?,
            Scalar::Paved(table, field) => table.declaration(field)?,
        };
        Ok(declaration)
    }

}

impl<'c, T: Struct> Scalar<'c, T> {

    pub fn ident(&self) -> Result<&'c syn::Ident> {
        let ident = match self {
            Scalar::Table(_, field) => field.ident(),
            Scalar::Paved(_, field) => field.ident(),
        };
        Ok(ident)
    }

    pub fn alias(&self) -> Result<String> {
        Ok(self.ident()?.unraw())
    }

}

macro_rules! base {
($table:ty, $field:ty) => {
both!($table, $field);

impl $table {

    pub fn table(&self) -> Result<Cow<str>> {
        let guard = cache::fetch();
        let table = match &self.attr.table.data.data {
            Paved::String(table) => return Ok(Cow::Borrowed(table)),
            Paved::Path(path) => path,
        };
        let table = guard.table(&table.try_into()?)?;
        let table = table.attr.table.data.data.clone();
        Ok(Cow::Owned(table))
    }

    pub fn returning(&self) -> Result<Option<&Returning>> {
        let data = self.attr.returning.as_ref().map(|name| {
            match &name.data {
                Some(info) => &info.data,
                None => const { &Returning::default() },
            }
        });
        Ok(data)
    }

    pub fn returnable(&self) -> Result<Returnable<Self>> {
        Ok(Returnable {
            table: self,
            paved: &self.attr.table.data,
            returning: self.returning()?,
        })
    }

    pub fn cells<'c, K, V, F, U, G>(
        &'c self,
        params: &mut Params<K, V, &'c $field>,
        mut val: F,
        mut wrap: G,
    ) -> Result<Vec<(&'c $field, V)>>
    where
        K: From<String> + Hash + Eq,
        V: Placer<&'c $field> + Clone,
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

    pub fn fields(&self) -> impl Iterator<Item = &$field> {
        self.fields.iter().filter(|field| {
            field.attr.skip.is_none()
        })
    }

    pub fn optional(&self, field: &$field) -> Option<Span> {
        match &field.attr.optional {
            Some(opt) => match &opt.data {
                Some(data) => data.data,
                None => true,
            }.then(|| opt.span),
            None => self.attr.optional.as_ref().and_then(|opt| {
                optype(&field.ty).map(|_| opt.span)
            }),
        }
    }

    pub fn dynamic(&self) -> Option<Span> {
        self.attr.dynamic.spany()
    }

    pub fn r#static(&self) -> Result<()> {
        let opt = self.fields.iter().find_map(|field| {
            self.optional(field)
        });
        r#static(self.dynamic(), opt)
    }

}

} }

impl DeleteTable {
    pub fn certain(&self) -> bool {
        self.attr.keyless.is_some() ||
        self.fields().any(|field| self.optional(field).is_none())
    }
}

impl InsertTable {
    pub fn certain(&self) -> bool {
        self.fields().any(|field| self.optional(field).is_none())
    }
}

impl SelectTable {
    pub fn certain(&self) -> bool { true }
}

impl UpdateTable {
    pub fn certain(&self) -> bool {
        let mut keys = self.fields().filter(|field| field.attr.key.is_some());
        let mut values = self.fields().filter(|field| field.attr.key.is_none());
        values.any(|field| self.optional(field).is_none()) && {
            keys.any(|field| self.optional(field).is_none()) ||
            self.attr.keyless.is_some()
        }
    }
}

impl QueryTable {
    pub fn formable(&self) -> bool {
        spany!(self.attr.from_row, self.attr.select).is_some()
    }
}



pub type Declaration<'c> = (Cow<'c, str>, &'c str);

pub trait Declare: Struct {
    fn declaration<'c>(&self, field: &'c Self::Field) -> Result<Declaration<'c>>;
    fn column<'c>(&self, field: &'c Self::Field) -> Result<Cow<'c, str>> {
        Ok(self.declaration(field)?.0)
    }
    #[allow(unused)]
    fn modifier<'c>(&self, field: &'c Self::Field) -> Result<&'c str> {
        Ok(self.declaration(field)?.1)
    }
}

pub trait Declared {
    fn declaration(&self) -> Result<Declaration>;
    fn column(&self) -> Result<Cow<str>> {
        Ok(self.declaration()?.0)
    }
    #[allow(unused)]
    fn modifier(&self) -> Result<&str> {
        Ok(self.declaration()?.1)
    }
}

macro_rules! both {
($table:ty, $field:ty) => {

impl $table {

    pub fn rename<'c>(&self, field: &$field, string: &'c str) -> Result<Cow<'c, str>> {
        let all = &self.attr.rename_all;
        let rename = &field.attr.rename;
        let renamed = match rename.as_ref().or(all.as_ref()) {
            Some(re) => re.data.data.rename(string),
            None => Cow::Borrowed(string),
        };
        Ok(renamed)
    }

    fn declare<'c>(&self, field: &'c $field, named: &syn::Ident) -> Result<Declaration<'c>> {
        let (name, info) = match &field.attr.column {
            None => {
                let ident = named.unraw();
                let name = self.rename(field, &ident)?;
                (Cow::Owned(name.into_owned()), "")
            }
            Some(column) => {
                let name = column.data.data.as_str();
                let (name, info) = {
                    if let Some(cut) = name.rfind('\0') {
                        (&name[..cut], &name[cut + 1..])
                    } else {
                        match name.find(['!', '?', ':']) {
                            Some(i) => name.split_at(i),
                            None => (name, ""),
                        }
                    }
                };
                let name = self.rename(field, name)?;
                (name, info)
            }
        };
        let info = match (info.chars().next(), &field.attr.infer) {
            (Some('!'), Some(_)) => "!: _",
            (Some('?'), Some(_)) => "?: _",
            (_, Some(_)) => ": _",
            (_, None) => info,
        };
        Ok((name, info))
    }

}

impl Declare for $table {
    fn declaration<'c>(&self, field: &'c $field) -> Result<Declaration<'c>> {
        self.declare(field, &field.ident)
    }
}

impl $table {

    #[cfg(not(feature = "checked"))]
    pub fn unchecked(&self) -> bool {
        true
    }

    #[cfg(feature = "checked")]
    pub fn unchecked(&self) -> bool {
        let data = self.attr.unchecked.as_ref().map(|data| {
            data.data.as_ref().map(|data| data.data)
        });
        match data {
            None => false,
            Some(None) => true,
            Some(Some(Checks::Query)) => true,
            Some(Some(Checks::Types)) => false,
        }
    }

    pub fn untyped(&self) -> bool {
        let data = self.attr.unchecked.as_ref().map(|data| {
            data.data.as_ref().map(|data| data.data)
        });
        match data {
            None => false,
            Some(None) => false,
            Some(Some(Checks::Query)) => false,
            Some(Some(Checks::Types)) => true,
        }
    }

}

} }



both!(QueryTable, QueryField);
base!(DeleteTable, DeleteField);
base!(InsertTable, InsertField);
base!(SelectTable, SelectField);
base!(UpdateTable, UpdateField);
