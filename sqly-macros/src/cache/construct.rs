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
    pub optional: Option<syn::Path>,
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
    pub optional: Option<&'c syn::Path>,
}

pub type Constructed<'c> = Column<'c, Construct<'c>>;



impl QueryTable {

    pub fn optional<'c>(&self, field: &'c QueryField) -> Result<Option<&'c syn::Path>> {
        Ok(optype(&field.ty).map(|optional| optional.0))
    }

    pub fn foreign<'c>(&self, field: &'c QueryField) -> Result<Option<Foreign<'c>>> {
        if field.attr.foreign.is_empty() {
            return Ok(None);
        }

        let optional = optype(&field.ty);

        let ty = match optional {
            Some((_, ty)) => ty,
            None => &field.ty,
        };

        let path = match typath(ty) {
            Some(path) => path,
            None => {
                let span = syn::spanned::Spanned::span(ty);
                let msg = "invalid type: not a path\n\
                    note: expected due to #[sqly(foreign)] attribute";
                return Err(syn::Error::new(span, msg));
            }
        };

        let optional = optional.map(|(path, _)| argone(path));

        Ok(Some(Foreign {
            optional,
            path,
        }))
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
                code
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
                },
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

    fn flatten(&'c self, opt: Option<&'c syn::Path>) -> Result<impl Iterator<Item = Result<Flattened<'c>>>> {
        let columns = self.fields.iter().map(move |column| {
            let once = std::iter::once(Ok(Flattened { column, construct: self, optional: opt }));
            let iter: Box<dyn Iterator<Item = _>> = match &column.code {
                Code::Skip => Box::new(once),
                Code::Query => Box::new(once),
                Code::Foreign(construct) => {
                    let opt = construct.optional()?.or(opt);
                    Box::new(once.chain(construct.flatten(opt)?))
                }
            };
            Ok(iter)
        }).flat_map(|iter| match iter {
            Err(err) => Box::new(std::iter::once(Err(err))),
            Ok(iter) => iter,
        });
        Ok(columns)
    }

    pub fn flattened(&'c self) -> Result<impl Iterator<Item = Result<Flattened<'c>>>> {
        self.flatten(None)
    }

}



impl<'c> Constructed<'c> {

    pub fn named(&self) -> Result<syn::Ident> {
        let named = match &self.code {
            Code::Foreign(construct) => {
                match &self.field.attr.foreign_named {
                    Some(named) => named.data.data.clone(),
                    None => match &self.field.attr.column {
                        Some(column) => {
                            let column = column.data.data.to_snake_case();
                            quote::format_ident!("r#{column}")
                        },
                        None => {
                            let prefix = &self.field.ident;
                            let suffix = match &construct.correlate(self)? {
                                Resolved::Attr(attr) => attr.column.to_snake_case(),
                                Resolved::Field(field) => field.named()?.to_string(),
                            };
                            quote::format_ident!("{prefix}_{suffix}")
                        }
                    }
                }
            },
            Code::Query => self.field.ident.clone(),
            Code::Skip => self.field.ident.clone(),
        };
        Ok(named)
    }

    pub fn typed(&'c self) -> Result<&'c syn::Type> {
        let typed = match &self.code {
            Code::Foreign(construct) => {
                match &self.field.attr.foreign_typed {
                    Some(typed) => &typed.data.data,
                    None => match &construct.correlate(self)? {
                        Resolved::Field(field) => field.typed()?,
                        Resolved::Attr(compromise) => {
                            let key  = &compromise.column;
                            let ident  = &compromise.construct.table.ident;
                            let span = self.field.ident.span();
                            let msg = format!("missing attribute: #[sqly(foreign_typed)]\n\
                                note: type unknown since \"{key}\" does not match any columns in {ident}");
                            return Err(syn::Error::new(span, msg));
                        }
                    }
                }
            },
            Code::Query => &self.field.ty,
            Code::Skip => &self.field.ty,
        };
        Ok(typed)
    }

}



impl<'c> Construct<'c> {

    pub fn correlate(&'c self, foreign: &'c Constructed<'c>) -> Result<Resolved<'c>> {
        let mut fields = self.fields.iter().filter(|column| {
            let skipped = column.table.skipped(column.field, Skips::Query);
            !skipped && match &foreign.field.attr.foreign_key {
                Some(key) => match &key.data.data {
                    Named::Ident(ident) => column.field.ident.eq(ident),
                    Named::String(string) => match column.column() {
                        Ok(column) => string.eq(&column),
                        Err(_) => false,
                    }
                },
                None => column.field.attr.key.is_some(),
            }
        });

        let first = fields.next();
        let field = match fields.next() {
            Some(_) => None,
            None => first,
        };

        let resolved = match field {
            Some(column) => Some(Resolved::Field(&column)),
            None => match &foreign.field.attr.foreign_key {
                Some(key) => match &key.data.data {
                    Named::String(column) => {
                        let compromise = Compromise {
                            construct: self,
                            foreign,
                            column,
                        };
                        Some(Resolved::Attr(compromise))
                    },
                    _ => None,
                },
                _ => None,
            }
        };

        let resolved = match resolved {
            Some(resolved) => resolved,
            None => {
                let ident = &self.table.ident;
                match &foreign.field.attr.foreign_key {
                    None => {
                        let span = foreign.field.ident.span();
                        let msg = match first {
                            None => format!("missing foreign key: no keys in {ident}\n\
                                help: use #[sqly(foreign_key = )] to disambiguate"),
                            _ => format!("ambiguous foreign key: multiple keys in {ident}\n\
                                help: use #[sqly(foreign_key = )] to disambiguate"),
                        };
                        return Err(syn::Error::new(span, msg));
                    },
                    Some(key) => {
                        let span = key.data.span;
                        let data = &key.data.data;
                        let msg = match first {
                            None => format!("unknown foreign key: {data} has no matches in {ident}\n\
                                help: use #[sqly(foreign_key = \"column_name\")] to join arbitrary columns"),
                            _ => format!("ambiguous foreign key: {data} has multiple matches in {ident}\n\
                                help: use #[sqly(foreign_key = field_ident)] to disambiguate matched fields"),
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
                if column.table.optional(column.field)?.is_none() {
                    match &column.field.attr.key {
                        Some(keys) => match keys.data.len() {
                            0 => id = id.or(Some(column)),
                            _ => key = key.or(Some(column)),
                        },
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
                if construct.optional()?.is_none() {
                    if let Some(column) = construct.constitute()? {
                        return Ok(Some(column));
                    }
                }
            }
        }

        Ok(None)
    }

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



impl<'c> Construct<'c> {

    pub fn params(&'c self) -> Result<Params<'c>> {
        let mut list = Vec::new();
        let mut params = Params::new();
        self.pave(&mut list, Vec::new())?;

        for item in &list {
            let path = item.contract(&list)?;
            if path.is_empty() {
                continue;
            }
            let local = path.join("__");
            let unique = item.location.unique()?;
            params.0.insert(local, unique);
        }

        params.emplace("table", self.unique()?);
        Ok(params)
    }

}

impl<'c> Flattened<'c> {

    pub fn foreigns(&self, foreign: &'c Construct<'c>) -> Result<Params<'c>> {
        let mut params = self.construct.params()?;
        let inner = match self.optional {
            Some(_) => ("left", "LEFT"),
            None => ("inner", "INNER"),
        };
        params.emplace("left", "left");
        params.emplace("LEFT", "LEFT");
        params.emplace("inner", inner.0);
        params.emplace("INNER", inner.1);
        params.emplace("other", foreign.unique()?);
        Ok(params)
    }

}



struct Path<'c> {
    segments: Vec<String>,
    location: &'c Construct<'c>,
}

impl<'c> Path<'c> {

    fn contract(&self, list: &[Path<'c>]) -> Result<Vec<String>> {
        let src = self.segments.as_slice();
        let mut dst = Vec::new();

        for i in 1..src.len() {
            let end = &src[i..];
            let seg = &src[i - 1];
            let mut dup = list.iter().filter(|path| {
                let x = match path.segments.len().checked_sub(end.len() + 1) {
                    Some(i) => path.segments[i].ne(seg),
                    None => true,
                };
                x && path.segments.ends_with(end)
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



use std::collections::{HashMap as Map};

pub struct Params<'c>(Map<String, &'c str>);

impl<'c> Params<'c> {

    pub fn new() -> Self {
        Self(Map::new())
    }

    pub fn emplace(&mut self, key: &str, val: &'c str) {
        if let Some(res) = self.0.insert(key.to_string(), val) {
            self.emplace(&format!("table__{key}"), res);
        }
    }

    pub fn replace(&self, src: &str, span: proc_macro2::Span) -> Result<String> {
        let mut res = String::new();
        let mut src = src;

        while let Some(i) = src.find('$') {
            let mut chars = src[i..].chars();
            let next = chars.nth(1);

            if next == Some('$') {
                res.push_str(&src[..=i]);
                src = &src[i + 2..];
                continue;
            }

            let var = match next {
                Some('{') => {
                    let j = match src[i + 2..].find('}') {
                        Some(j) => j + i + 2,
                        None => {
                            let msg = "unmatched opening brace: \"${\" expects a closing \"}\"\n\
                                help: use \"$${\" to escape and resolve to the literal \"${\"";
                            return Err(syn::Error::new(span, msg));
                        }
                    };
                    let var = &src[i + 2..j];
                    res.push_str(&src[..i]);
                    src = &src[j + 1..];
                    var
                },
                Some(char) => {
                    let o = if char == 'r' && chars.next() == Some('#') { 3 } else { 1 };
                    let j = src[i + o..].find(|c| !unicode_ident::is_xid_continue(c));
                    let j = j.map_or(src.len(), |j| j + i + o);
                    let var = &src[i + 1..j];
                    res.push_str(&src[..i]);
                    src = &src[j..];
                    var
                },
                None => {
                    let var = &src[i + 1..];
                    src = &src[i + 1..];
                    var
                }
            };

            if var.chars().all(|c| c.is_whitespace()) {
                let msg = match next {
                    Some('{') => format!("missing identifier: \"${{{var}}}\" is expected to enclose an identifier\n\
                        help: use \"$${{{var}}}\" to escape and resolve to the literal \"${{{var}}}\""),
                    Some(char) => format!("missing identifier: \"$\" is expected to precede an identifier\n\
                        help: use \"$${char}\" to escape and resolve to the literal \"${char}\""),
                    None => format!("missing identifier: \"$\" is expected to precede an identifier\n\
                        help: use \"$$\" to escape and resolve to the literal \"$\""),
                };
                return Err(syn::Error::new(span, msg));
            }

            let ident = match syn::parse_str::<syn::Ident>(var) {
                Ok(ident) => {
                    let ident = ident.to_string();
                    match ident.strip_prefix("r#") {
                        Some(strip) => strip.to_string(),
                        None => ident,
                    }
                },
                Err(_) => {
                    let msg = match next.unwrap_or('\0') {
                        '{' => format!("invalid identifier: \"{var}\"\n\
                            help: use \"$${{{var}}}\" to escape and resolve to the literal \"${{{var}}}\""),
                        _ => format!("invalid identifier: \"{var}\"\n\
                            help: use \"$${var}\" to escape and resolve to the literal \"${var}\""),
                    };
                    return Err(syn::Error::new(span, msg));
                }
            };

            match self.0.get(&ident) {
                Some(val) => res.push_str(val),
                None => {
                    let mut params = self.0.keys().map(|key| key.as_str()).collect::<Vec<_>>();
                    params.sort_unstable_by_key(|params| (params.len(), params.to_string()));
                    let params = params.join(", ");
                    let msg = match next.unwrap_or('\0') {
                        '{' => format!("unknown paramseter: {var}\n \
                            known paramsaters: {params}\n\
                            help: use \"$${{{var}}}\" to escape and resolve to the literal \"${{{var}}}\""),
                        _ => format!("unknown paramseter: {var}\n \
                            known paramsaters: {params}\n\
                            help: use \"$${var}\" to escape and resolve to the literal \"${var}\""),
                    };
                    return Err(syn::Error::new(span, msg));
                }
            }
        }

        res.push_str(src);
        Ok(res)
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    mod replace {
        use super::*;

        fn result(src: &str) -> Result<String> {
            let mut params = Params::new();
            params.0.insert("one".into(), "1");
            params.0.insert("two".into(), "2");
            params.0.insert("mod".into(), "mod");
            params.0.insert("SELF".into(), "self");
            params.0.insert("inner".into(), "LEFT");
            params.0.insert("INNER".into(), "INNER");
            params.0.insert("table".into(), "elbat");
            params.0.insert("other".into(), "rehto");
            let span = proc_macro2::Span::call_site();
            params.replace(src, span)
        }

        fn replaced(src: &str) -> String {
            result(src).unwrap()
        }

        fn errored(err: &str, src: &str) {
            assert!(result(src).unwrap_err().to_string().contains(err));
        }

        #[test]
        fn empty() {
            assert_eq!(replaced(""), "");
        }

        #[test]
        fn copy() {
            assert_eq!(replaced("copy"), "copy");
        }

        #[test]
        fn replace() {
            assert_eq!(replaced("$one"), "1");
            assert_eq!(replaced("$two $one"), "2 1");
            assert_eq!(replaced("$two${ one }$two"), "212");
            assert_eq!(replaced("${one}$two${one}"), "121");
            assert_eq!(replaced("r#${r#table}#"), "r#elbat#");
            assert_eq!(replaced("{$r#other#}"), "{rehto#}");
            assert_eq!(replaced("{${SELF}}"), "{self}");
            assert_eq!(replaced("${ r#mod }"), "mod");
            assert_eq!(replaced("$r#mod"), "mod");
        }

        #[test]
        fn escape() {
            assert_eq!(replaced("$$"), "$");
            assert_eq!(replaced("$$$$r#"), "$$r#");
            assert_eq!(replaced("$$table"), "$table");
            assert_eq!(replaced("$$$table$$"), "$elbat$");
            assert_eq!(replaced("$one$$one$one"), "1$one1");
            assert_eq!(replaced("$${ table }"), "${ table }");
            assert_eq!(replaced("$${ $table }"), "${ elbat }");
            assert_eq!(replaced("$${ r#$$ "), "${ r#$ ");
            assert_eq!(replaced("$${"), "${");
        }

        #[test]
        fn statement() {
            assert_eq!(replaced(
                "$INNER JOIN other AS $other ON $other.id = $table.other_id"
            ), "INNER JOIN other AS rehto ON rehto.id = elbat.other_id");
            assert_eq!(replaced(
                r#"$inner JOIN other AS "${other}" ON $other.id="$table".other_id"#
            ), r#"LEFT JOIN other AS "rehto" ON rehto.id="elbat".other_id"#);
            assert_eq!(replaced(
                "$inner JOIN other_a AS ${other}_a ON ${other}_a.id = $table.other_a_id\n\
                $inner JOIN other_b AS ${other}_b ON ${other}_b.id = $table.other_b_id\n\
                $INNER JOIN other AS $other ON\n\
                    $other.id_a = ${other}_a.id OR\n\
                    $other.id_b = ${other}_b.id"
            ), "LEFT JOIN other_a AS rehto_a ON rehto_a.id = elbat.other_a_id\n\
                LEFT JOIN other_b AS rehto_b ON rehto_b.id = elbat.other_b_id\n\
                INNER JOIN other AS rehto ON\n\
                    rehto.id_a = rehto_a.id OR\n\
                    rehto.id_b = rehto_b.id"
            );
        }

        #[test]
        fn missing() {
            let err = "missing";
            errored(err, "$");
            errored(err, "$ $");
            errored(err, "$ {");
            errored(err, "$$$");
            errored(err, "$${$}");
            errored(err, "$one$");
            errored(err, "${   }");
            errored(err, "r#${}");
            errored(err, "$ r#");
        }

        #[test]
        fn unmatched() {
            let err = "unmatched";
            errored(err, "${");
            errored(err, "${$");
            errored(err, "${${");
            errored(err, "$$${");
            errored(err, "$${}${");
            errored(err, "$one${");
            errored(err, "${$one");
            errored(err, "${r#{r");
            errored(err, "${r#");
        }

        #[test]
        fn invalid() {
            let err = "invalid";
            errored(err, "$_");
            errored(err, "$1");
            errored(err, "$mod");
            errored(err, "$r#self");
            errored(err, "${ $one }");
            errored(err, "${two one}");
            errored(err, "$r#{r}");
            errored(err, "${r#}");
            errored(err, "$r#");
        }

        #[test]
        fn unknown() {
            let err = "unknown";
            errored(err, "$a");
            errored(err, "$a1");
            errored(err, "$_one");
            errored(err, "$two_");
            errored(err, "${ r#b }");
            errored(err, "${ r#_2 }");
            errored(err, "$Table");
            errored(err, "$eblat");
            errored(err, "${__}");
        }

    }

}
