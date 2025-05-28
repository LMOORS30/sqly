use std::cell::OnceCell;

use heck::ToSnakeCase;

use super::*;



pub enum Code<C> {
    Foreign(C),
    Query,
    Skip,
}

pub struct Column<'c, C> {
    pub unique: OnceCell<String>,
    pub table: &'c QueryTable,
    pub field: &'c QueryField,
    pub code: Code<C>,
}

pub struct Construct<'c> {
    pub table: &'c QueryTable,
    pub fields: Vec<Constructed<'c>>,
    pub foreign: Option<Foreign<'c>>,
    pub unique: OnceCell<String>,
}

pub struct Foreign<'c> {
    pub path: &'c syn::Path,
    pub nullable: Option<Nullable<'c>>,
}

#[derive(Clone, Copy)]
pub enum Nullable<'c> {
    Default(Span, Option<&'c syn::Expr>),
    Option(&'c syn::Path),
}

pub enum Resolved<'c> {
    Attr(Compromise<'c>),
    Field(&'c Constructed<'c>),
}

pub struct Compromise<'c> {
    pub construct: &'c Construct<'c>,
    pub foreign: &'c Constructed<'c>,
    pub column: &'c str,
}

pub struct Flattened<'c> {
    pub column: &'c Constructed<'c>,
    pub construct: &'c Construct<'c>,
    pub nullable: Option<Nullable<'c>>,
    pub level: usize,
}

pub type Constructed<'c> = Column<'c, Construct<'c>>;

pub struct Returnable<'c, T> {
    pub table: &'c T,
    pub paved: &'c Info<Paved>,
    pub returning: Option<Cow<'c, Returning>>,
}

pub enum Scalar<'c, T: Struct> {
    Table(&'c T, &'c T::Field),
    Paved(&'c QueryTable, &'c QueryField),
}

pub enum Returns<'c, T: Struct> {
    None,
    Scalar(Scalar<'c, T>),
    Tuple(Vec<Scalar<'c, T>>),
    Table(&'c syn::Path, &'c QueryTable),
    Construct(&'c syn::Path, &'c Construct<'c>),
}



impl QueryTable {

    pub fn defaulted<'c>(&'c self, field: &'c QueryField) -> Result<Option<Nullable<'c>>> {
        let opt = match &field.attr.default {
            Some(default) => {
                let expr = default.data.as_ref();
                let expr = expr.map(|data| &data.data);
                Some(Nullable::Default(default.span, expr))
            }
            None => None,
        };
        Ok(opt)
    }

    pub fn nullable<'c>(&'c self, field: &'c QueryField) -> Result<Option<Nullable<'c>>> {
        let opt = match self.defaulted(field)? {
            Some(defaulted) => Some(defaulted),
            None => {
                let opt = optype(self.ty(field)?);
                let path = opt.map(|(path, _)| path);
                path.map(Nullable::Option)
            }
        };
        Ok(opt)
    }

    pub fn foreign<'c>(&'c self, field: &'c QueryField) -> Result<Option<Foreign<'c>>> {
        if field.attr.foreign.is_empty() {
            return Ok(None);
        }
        let ty = self.ty(field)?;
        let ty = match optype(ty) {
            Some((_, ty)) => ty,
            None => ty,
        };
        let path = match typath(ty) {
            Some(path) => path,
            None => {
                let msg = "invalid type: expected path\n\
                    note: expected due to #[sqly(foreign)] attribute";
                return Err(syn::Error::new_spanned(ty, msg));
            }
        };
        let nullable = self.nullable(field)?;
        Ok(Some(Foreign { nullable, path }))
    }

    pub fn coded<'c>(&'c self) -> Result<impl Iterator<Item = Result<Column<'c, Foreign<'c>>>>> {
        let coded = self.fields.iter().map(move |field| {
            let code = {
                if self.skipped(field, Skips::FromRow) { Code::Skip }
                else if let Some(foreign) = self.foreign(field)? {
                    Code::Foreign(foreign)
                }
                else { Code::Query }
            };
            Ok(Column {
                unique: OnceCell::new(),
                table: self,
                field,
                code,
            })
        });
        Ok(coded)
    }

}



impl QueryTable {

    pub fn colocate<'c>(&'c self, local: &'c mut Local) -> Result<&'c Local> {
        let guard = cache::fetch();
        for column in self.coded()? {
            if let Code::Foreign(foreign) = column?.code {
                let id = Id::try_from(foreign.path)?;
                if !local.has_table(&id) {
                    let table = guard.table(&id)?.sync()?;
                    local.put_table(id.clone(), table)?;
                }
                let (id, table) = local.pop_table(&id)?;
                table.colocate(local)?;
                local.put_table(id, table)?;
            }
        }
        Ok(&*local)
    }

    fn locolate<'c>(&'c self, local: &'c Local) -> Result<Construct<'c>> {
        let fields = self.coded()?.map(|column| {
            let column = column?;
            let code = match column.code {
                Code::Foreign(foreign) => {
                    let id = &Id::try_from(foreign.path)?;
                    let table = local.get_table(id)?;
                    let mut construct = table.locolate(local)?;
                    construct.foreign = Some(foreign);
                    Code::Foreign(construct)
                }
                Code::Query => Code::Query,
                Code::Skip => Code::Skip,
            };
            Ok(Column {
                unique: column.unique,
                table: column.table,
                field: column.field,
                code,
            })
        }).collect::<Result<Vec<_>>>()?;
        Ok(Construct {
            unique: OnceCell::new(),
            foreign: None,
            table: self,
            fields,
        })
    }

    pub fn construct<'c>(&'c self, local: &'c Local) -> Result<Construct<'c>> {
        let construct = self.locolate(local)?;
        construct.contract()?;
        Ok(construct)
    }

}



impl<'c> Construct<'c> {

    fn flattened(&'c self, opt: Option<Nullable<'c>>, n: usize)
        -> Result<impl Iterator<Item = Result<Flattened<'c>>>>
    {
        let flatten = self.fields.iter().map(move |column| {
            let once = std::iter::once(Ok(Flattened {
                construct: self,
                nullable: opt,
                level: n,
                column,
            }));
            let iter: Box<dyn Iterator<Item = _>> = match &column.code {
                Code::Skip => Box::new(once),
                Code::Query => Box::new(once),
                Code::Foreign(construct) => {
                    let opt = construct.nullable()?.or(opt);
                    Box::new(once.chain(construct.flattened(opt, n + 1)?))
                }
            };
            Ok(iter)
        }).flat_map(|iter| match iter {
            Err(err) => Box::new(std::iter::once(Err(err))),
            Ok(iter) => iter,
        });
        Ok(flatten)
    }

    pub fn flatten(&'c self) -> Result<impl Iterator<Item = Result<Flattened<'c>>>> {
        self.flattened(None, 0)
    }

}



impl<'c> Constructed<'c> {

    pub fn renamed(&'c self) -> Result<Cow<'c, syn::Ident>> {
        let renamed = match &self.field.attr.named {
            Some(named) => Cow::Borrowed(&named.data.data),
            None => match &self.code {
                Code::Foreign(construct) => {
                    match &self.field.attr.column {
                        Some(column) => {
                            let ident = column.data.data.to_snake_case();
                            let mut ident = quote::format_ident!("r#{ident}");
                            ident.set_span(column.data.span());
                            Cow::Owned(ident)
                        }
                        None => {
                            let prefix = &self.field.ident;
                            let mut ident = match &construct.correlate(self)? {
                                Resolved::Attr(attr) => {
                                    let suffix = attr.column.to_snake_case();
                                    quote::format_ident!("{prefix}_{suffix}")
                                }
                                Resolved::Field(field) => {
                                    let suffix = field.renamed()?;
                                    quote::format_ident!("{prefix}_{suffix}")
                                }
                            };
                            ident.set_span(prefix.span());
                            Cow::Owned(ident)
                        }
                    }
                }
                Code::Query => Cow::Borrowed(&self.field.ident),
                Code::Skip => Cow::Borrowed(&self.field.ident),
            }
        };
        Ok(renamed)
    }

    pub fn typed(&'c self) -> Result<Cow<'c, syn::Type>> {
        let typed = match &self.code {
            Code::Foreign(construct) => {
                match &self.field.attr.typed {
                    Some(typed) => Cow::Borrowed(&typed.data.data),
                    None => match &construct.correlate(self)? {
                        Resolved::Field(field) => field.typed()?,
                        Resolved::Attr(compromise) => {
                            let span = match self.field.attr.target.spany() {
                                None => self.field.ident.span(),
                                Some(span) => span,
                            };
                            let key = &compromise.column;
                            let ident = &compromise.construct.table.ident;
                            let msg = format!("missing attribute: #[sqly(typed)]\n\
                                note: type unknown since \"{key}\" does not identify a column in {ident}");
                            return Err(syn::Error::new(span, msg));
                        }
                    }
                }
            }
            Code::Query => self.table.typed(self.field)?,
            Code::Skip => self.table.typed(self.field)?,
        };
        Ok(typed)
    }

    pub fn retyped(&'c self, r#type: Types) -> Result<Cow<'c, syn::Type>> {
        let retyped = match &self.field.attr.typed {
            Some(typed) => Cow::Borrowed(&typed.data.data),
            None => {
                let mut ty = self.typed()?;
                if let Some(span) = self.table.optional(self.field, r#type) {
                    ty = Cow::Owned(syn::parse_quote_spanned!(span => ::core::option::Option<#ty>));
                }
                ty
            }
        };
        Ok(retyped)
    }

}



impl<'c> Construct<'c> {

    pub fn correlate(&'c self, foreign: &'c Constructed<'c>) -> Result<Resolved<'c>> {
        let mut fields = self.fields.iter().filter(|column| {
            match &foreign.field.attr.target {
                Some(target) => match &target.data.data {
                    Named::Ident(ident) => column.field.ident.unraw().eq(&ident.unraw()),
                    Named::String(string) => match column.column() {
                        Ok(column) => string.eq(&column),
                        Err(_) => false,
                    }
                }
                None => column.field.attr.key.is_some(),
            }
        });
        let first = fields.next();
        let field = match fields.next() {
            Some(_) => None,
            None => first,
        };
        let resolved = match field {
            Some(column) => Some(Resolved::Field(column)),
            None => match &foreign.field.attr.target {
                Some(target) => match &target.data.data {
                    Named::String(column) => {
                        let compromise = Compromise {
                            construct: self,
                            foreign,
                            column,
                        };
                        Some(Resolved::Attr(compromise))
                    }
                    _ => None,
                }
                _ => None,
            }
        };
        let resolved = match resolved {
            Some(resolved) => resolved,
            None => {
                let ident = &self.table.ident;
                match &foreign.field.attr.target {
                    None => {
                        let span = match foreign.field.attr.foreign.spany() {
                            None => foreign.field.ident.span(),
                            Some(span) => span,
                        };
                        let msg = match first {
                            None => format!("missing target: no keys in {ident}\n\
                                help: use #[sqly(target)] to disambiguate"),
                            _ => format!("ambiguous target: multiple keys in {ident}\n\
                                help: use #[sqly(target)] to disambiguate"),
                        };
                        return Err(syn::Error::new(span, msg));
                    }
                    Some(target) => {
                        let data = &target.data.data;
                        let msg = match first {
                            None => format!("unknown target: {data} has no matches in {ident}\n\
                                help: use #[sqly(target = \"column_name\")] to join arbitrary columns"),
                            _ => format!("ambiguous target: {data} has multiple matches in {ident}\n\
                                help: use #[sqly(target = field_ident)] to disambiguate matched fields"),
                        };
                        return Err(syn::Error::new(target.data.span(), msg));
                    }
                }
            }
        };
        Ok(resolved)
    }

    pub fn constitute(&'c self) -> Result<Option<&'c Constructed<'c>>> {
        let mut id = None;
        let mut key = None;
        let mut rest = None;
        for column in &self.fields {
            if let Code::Query = &column.code {
                if column.table.nullable(column.field)?.is_none() {
                    match &column.field.attr.key {
                        Some(keys) => match keys.data.len() {
                            0 => id = id.or(Some(column)),
                            _ => key = key.or(Some(column)),
                        }
                        _ => rest = rest.or(Some(column)),
                    }
                }
            }
        }
        if let Some(column) = id.or(key).or(rest) {
            return Ok(Some(column));
        }
        for column in &self.fields {
            if let Code::Foreign(construct) = &column.code {
                if construct.nullable()?.is_none() {
                    if let Some(column) = construct.constitute()? {
                        return Ok(Some(column));
                    }
                }
            }
        }
        Ok(None)
    }

}



impl<'c, T: Struct> Returnable<'c, T> {

    pub fn companion(&self) -> Result<Option<&syn::Path>> {
        let companion = match &self.returning {
            Some(returning) => match &returning.table {
                Some(table) => Some(table),
                None => match &self.paved.data {
                    Paved::Path(table) => Some(table),
                    Paved::String(_) => None,
                }
            }
            None => None,
        };
        Ok(companion)
    }

    pub fn colocate(&'c self, local: &'c mut Local) -> Result<&'c Local> {
        if let Some(path) = self.companion()? {
            let guard = cache::fetch();
            let id = Id::try_from(path)?;
            if !local.has_table(&id) {
                let table = guard.table(&id)?.sync()?;
                local.put_table(id, table)?;
            }
        }
        Ok(&*local)
    }

    pub fn correlate(&self, local: &'c Local, field: &syn::Ident) -> Result<Scalar<'c, T>> {
        let ident = field.unraw();
        let mut absent = Vec::new();
        if let Some(returning) = &self.returning {
            if returning.table.is_none() {
                for field in self.table.fields() {
                    if field.ident().unraw().eq(&ident) {
                        return Ok(Scalar::Table(self.table, field));
                    }
                }
                absent.push(self.table.ident().to_string());
            }
        }
        if let Some(path) = self.companion()? {
            let table = local.get_table(&path.try_into()?)?;
            for column in table.coded()? {
                let column = column?;
                if column.field.ident().unraw().eq(&ident) {
                    if let Code::Foreign(_) = column.code {
                        let msg = "invalid field: cannot return foreign field\n\
                            note: #[sqly(returning)] requires a field without #[sqly(foreign)]";
                        return Err(syn::Error::new_spanned(field, msg));
                    }
                    return Ok(Scalar::Paved(column.table, column.field));
                }
            }
            absent.push(table.ident().to_string());
        }
        let msg = format!("unknown field: identifier not present in {}", absent.join(" or "));
        return Err(syn::Error::new_spanned(field, msg));
    }

    pub fn returns(&'c self, local: &'c Local) -> Result<Returns<'c, T>> {
        let returning = match &self.returning {
            None => return Ok(Returns::None),
            Some(returning) => returning,
        };
        if !returning.fields.is_empty() {
            let iter = returning.fields.iter();
            let mut list = iter.map(|field| {
                self.correlate(local, field)
            }).collect::<Result<Vec<_>>>()?;
            let returns = match list.len() {
                0 | 1 => match list.pop() {
                    None => {
                        let msg = "no returning fields found";
                        return Err(syn::Error::new(Span::call_site(), msg));
                    }
                    Some(item) => Returns::Scalar(item),
                }
                _ => Returns::Tuple(list),
            };
            return Ok(returns);
        }
        let (path, table) = match self.companion()? {
            Some(path) => (path, local.get_table(&path.try_into()?)?),
            None => {
                let msg = "invalid table identifier: expected path\n\
                    note: #[sqly(returning)] requires a struct with #[derive(Table)]";
                return Err(syn::Error::new(self.paved.span(), msg));
            }
        };
        for column in table.coded()? {
            if let Code::Foreign(_) = column?.code {
                let msg = "invalid table: cannot return foreign table\n\
                    note: #[sqly(returning)] requires a table without #[sqly(foreign)]";
                return Err(syn::Error::new_spanned(path, msg));
            }
        }
        if !table.formable() {
            let msg = "missing attribute: the referenced table must have #[sqly(from_row)]\n\
                note: #[sqly(returning)] uses the sqlx::FromRow definition for its query";
            return Err(syn::Error::new_spanned(path, msg));
        };
        Ok(Returns::Table(path, table))
    }

}



type Link = Option<Rc<Part>>;

struct Part {
    part: String,
    prev: Link,
}

struct Path<'c> {
    place: &'c Construct<'c>,
    parts: Link,
}

impl<'c> Construct<'c> {

    fn pave(&'c self, list: &mut Vec<Path<'c>>, link: Link) -> Result<()> {
        for column in self.fields.iter() {
            if let Code::Foreign(construct) = &column.code {
                construct.pave(list, Some(Rc::new(Part {
                    part: column.field.ident.unraw(),
                    prev: link.clone(),
                })))?;
            }
        }
        list.push(Path {
            place: self,
            parts: link,
        });
        Ok(())
    }

    fn contract(&'c self) -> Result<()> {
        let mut list = Vec::new();
        self.pave(&mut list, None)?;

        for item in &list {
            let path = item.contract(&list)?;
            let unique = match path.len() {
                0 => "self".to_string(),
                _ => path.join("__"),
            };
            let cell = &item.place.unique;
            let unique = cell.get_or_init(|| unique);
            for column in item.place.fields.iter() {
                let part = column.field.ident.unraw();
                let path = if list.len() <= 1 { part } else {
                    [unique.as_str(), part.as_str()].join("__")
                };
                column.unique.get_or_init(|| path);
            }
        }

        Ok(())
    }

}

impl<'c> Path<'c> {

    fn contract(&'c self, list: &[Path<'c>]) -> Result<Vec<&'c str>> {
        let mut res = Vec::new();
        let mut cur = match &self.parts {
            None => return Ok(res),
            Some(last) => last,
        };

        fn step<'c>(item: &'c Link, step: &str) -> Option<&'c Link> {
            item.as_ref().and_then(|link| {
                match link.part == step {
                    true => Some(&link.prev),
                    false => None,
                }
            })
        }

        let mut dup = list.iter().filter_map(|item| {
            step(&item.parts, &cur.part)
        }).collect::<Vec<_>>();
        res.push(&cur.part);

        while let Some(link) = &cur.prev {
            let len = dup.len();
            if len <= 1 { break; }
            dup.retain_mut(|item| {
                let step = step(item, &link.part);
                step.is_some_and(|step| {
                    *item = step;
                    true
                })
            });
            if dup.len() < len {
                res.push(&link.part);
            }
            cur = link;
        }

        res.reverse();
        Ok(res)
    }

}

impl<'c> Construct<'c> {

    pub fn params(&'c self) -> Result<Params<String, &'c str>> {
        let mut list = Vec::new();
        let mut params = Params::default();
        self.pave(&mut list, None)?;

        for item in &list {
            let path = item.contract(&list)?;
            if path.is_empty() {
                continue;
            }
            let unique = item.place.unique()?;
            let local = path.join("__");
            params.put(local, unique);
        }

        params.displace("table", self.unique()?);
        Ok(params)
    }

}

impl<'c> Flattened<'c> {

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
