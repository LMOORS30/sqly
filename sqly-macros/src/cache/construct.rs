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
    Default(Option<&'c syn::Path>),
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



impl QueryTable {

    pub fn defaulted<'c>(&'c self, field: &'c QueryField) -> Result<Option<Nullable<'c>>> {
        let opt = match &field.attr.default {
            Some(default) => {
                let path = default.data.as_ref();
                let path = path.map(|data| &data.data);
                Some(Nullable::Default(path))
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
                let span = ty.span();
                let msg = "invalid type: not a path\n\
                    note: expected due to #[sqly(foreign)] attribute";
                return Err(syn::Error::new(span, msg));
            }
        };

        let nullable = self.nullable(field)?;
        Ok(Some(Foreign { nullable, path }))
    }

    pub fn coded<'c>(&'c self) -> Result<impl Iterator<Item = Result<Column<'c, Foreign<'c>>>>> {
        let coded = self.fields.iter().map(move |field| {
            let code = {
                if self.skipped(field, Skips::Query) { Code::Skip }
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
            let column = column?;
            if let Code::Foreign(foreign) = column.code {
                let id = Id::try_from(foreign.path)?;
                let table = guard.table(&id)?.sync()?;
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

    fn flattened(&'c self, opt: Option<Nullable<'c>>, n: usize) -> Result<impl Iterator<Item = Result<Flattened<'c>>>> {
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

    pub fn renamed(&self) -> Result<syn::Ident> {
        let renamed = match &self.field.attr.named {
            Some(named) => named.data.data.clone(),
            None => match &self.code {
                Code::Foreign(construct) => {
                    match &self.field.attr.column {
                        Some(column) => {
                            let ident = column.data.data.to_snake_case();
                            let mut ident = quote::format_ident!("r#{ident}");
                            ident.set_span(column.data.span);
                            ident
                        }
                        None => {
                            let prefix = &self.field.ident;
                            let suffix = match &construct.correlate(self)? {
                                Resolved::Attr(attr) => attr.column.to_snake_case(),
                                Resolved::Field(field) => field.renamed()?.to_string(),
                            };
                            let mut ident = quote::format_ident!("{prefix}_{suffix}");
                            ident.set_span(prefix.span());
                            ident
                        }
                    }
                }
                Code::Query => self.field.ident.clone(),
                Code::Skip => self.field.ident.clone(),
            }
        };
        Ok(renamed)
    }

    pub fn typed(&'c self) -> Result<TokenStream> {
        let typed = match &self.code {
            Code::Foreign(construct) => {
                match &self.field.attr.typed {
                    Some(typed) => {
                        let typed = &typed.data.data;
                        quote::quote! { #typed }
                    }
                    None => match &construct.correlate(self)? {
                        Resolved::Field(field) => field.typed()?,
                        Resolved::Attr(compromise) => {
                            let key = &compromise.column;
                            let ident = &compromise.construct.table.ident;
                            let span = self.field.ident.span();
                            let msg = format!("missing attribute: #[sqly(typed)]\n\
                                note: type unknown since \"{key}\" does not match any columns in {ident}");
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

    pub fn retyped(&'c self) -> Result<TokenStream> {
        let retyped = match &self.field.attr.typed {
            Some(typed) => {
                let typed = &typed.data.data;
                quote::quote! { #typed }
            }
            None => self.typed()?,
        };
        Ok(retyped)
    }

}



impl<'c> Construct<'c> {

    pub fn correlate(&'c self, foreign: &'c Constructed<'c>) -> Result<Resolved<'c>> {
        let mut fields = self.fields.iter().filter(|column| {
            match &foreign.field.attr.target {
                Some(target) => match &target.data.data {
                    Named::Ident(ident) => column.field.ident.eq(ident),
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
                        let span = foreign.field.ident.span();
                        let msg = match first {
                            None => format!("missing target: no keys in {ident}\n\
                                help: use #[sqly(target)] to disambiguate"),
                            _ => format!("ambiguous target: multiple keys in {ident}\n\
                                help: use #[sqly(target)] to disambiguate"),
                        };
                        return Err(syn::Error::new(span, msg));
                    }
                    Some(target) => {
                        let span = target.data.span;
                        let data = &target.data.data;
                        let msg = match first {
                            None => format!("unknown target: {data} has no matches in {ident}\n\
                                help: use #[sqly(target = \"column_name\")] to join arbitrary columns"),
                            _ => format!("ambiguous target: {data} has multiple matches in {ident}\n\
                                help: use #[sqly(target = field_ident)] to disambiguate matched fields"),
                        };
                        return Err(syn::Error::new(span, msg));
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



struct Path<'c> {
    segments: Vec<String>,
    location: &'c Construct<'c>,
}

impl<'c> Construct<'c> {

    fn pave(&'c self, list: &mut Vec<Path<'c>>, path: Vec<String>) -> Result<()> {
        for column in self.fields.iter() {
            if let Code::Foreign(construct) = &column.code {
                let mut path = path.clone();
                path.push(column.segment()?);
                construct.pave(list, path)?;
            }
        }
        list.push(Path {
            location: self,
            segments: path,
        });
        Ok(())
    }

    fn contract(&self) -> Result<()> {
        let mut list = Vec::new();
        self.pave(&mut list, Vec::new())?;

        for item in &list {
            let path = item.contract(&list)?;
            let unique = match path.len() {
                0 => "self".to_string(),
                _ => path.join("__"),
            };
            let cell = &item.location.unique;
            let unique = cell.get_or_init(|| unique);
            for column in item.location.fields.iter() {
                let segment = column.segment()?;
                let path = [unique.as_str(), segment.as_str()];
                column.unique.get_or_init(|| path.join("__"));
            }
        }

        Ok(())
    }

}

impl<'c> Path<'c> {

    fn contract(&self, list: &[Path<'c>]) -> Result<Vec<String>> {
        let src = self.segments.as_slice();
        let mut dst = Vec::new();

        for i in 1..src.len() {
            let end = &src[i..];
            let seg = &src[i - 1];
            let mut dup = list.iter().filter(|path| {
                (match path.segments.len().checked_sub(end.len() + 1) {
                    Some(i) => path.segments[i].ne(seg),
                    None => true,
                }) && path.segments.ends_with(end)
            });
            if dup.next().is_some() {
                dst.push(seg.clone());
            }
        }

        if let Some(last) = src.last() {
            dst.push(last.clone());
        }

        Ok(dst)
    }

}

impl<'c> Construct<'c> {

    pub fn params(&'c self) -> Result<Params<String, &'c str>> {
        let mut list = Vec::new();
        let mut params = Params::default();
        self.pave(&mut list, Vec::new())?;

        for item in &list {
            let path = item.contract(&list)?;
            if path.is_empty() {
                continue;
            }
            let local = path.join("__");
            let unique = item.location.unique()?;
            params.put(local, unique);
        }

        params.displace("table", self.unique()?);
        Ok(params)
    }

}
