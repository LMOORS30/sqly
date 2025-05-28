use super::*;

mod delete;
mod insert;
mod select;
mod update;

pub use delete::*;
pub use insert::*;
pub use select::*;
pub use update::*;



vars! {
    pub Structs {
        (Flat = flat),
        (Delete = delete),
        (Insert = insert),
        (Select = select),
        (Update = update),
    }
    pub Skips {
        (Delete = delete),
        (Insert = insert),
        (Select = select),
        (Update = update),
        (FromRow = from_row),
    }
    pub Types: Structs, Skips {
        (Delete = delete),
        (Insert = insert),
        (Select = select),
        (Update = update),
    }
    pub Keys: Types {
        (Delete = delete),
        (Select = select),
        (Update = update),
    }
    pub Optionals {
        (Keys = keys),
        (Values = values),
    }
    pub Checks {
        (Query = query),
        (Types = types),
    }
    pub Print {
        (Warn = warn),
        (Panic = panic),
        (StdOut = stdout),
        (StdErr = stderr),
    }
    pub Rename {
        (None = "none"),
        (LowerCase = "lowercase"),
        (UpperCase = "UPPERCASE"),
        (CamelCase = "camelCase"),
        (PascalCase = "PascalCase"),
        (SnakeCase = "snake_case"),
        (KebabCase = "kebab-case"),
        (UpperSnakeCase = "SCREAMING_SNAKE_CASE"),
        (UpperKebabCase = "SCREAMING-KEBAB-CASE"),
    }
    pub Named {
        (String: String),
        (Ident: syn::Ident),
    }
    pub Paved {
        (String: String),
        (Path: syn::Path),
    }
}

safe! {
    pub struct Returning {
        pub table: syn::Path [?],
        pub fields: syn::Ident [*],
    }
}

parse! {
    pub QueryTable {
        ((table)! (= String)!),
        ((rename)? (= Rename)!),

        ((from_row)?),
        ((from_flat)?),
        ((flat_row)?),

        ((flat)? (= syn::Ident)?),
        ((delete)? (= syn::Ident)?),
        ((insert)? (= syn::Ident)?),
        ((select)? (= syn::Ident)?),
        ((update)? (= syn::Ident)?),

        ((flat_derive)* (= syn::Path)+),
        ((query_derive)* (= syn::Path)+),
        ((delete_derive)* (= syn::Path)+),
        ((insert_derive)* (= syn::Path)+),
        ((select_derive)* (= syn::Path)+),
        ((update_derive)* (= syn::Path)+),

        ((flat_visibility)? (= syn::Visibility)!),
        ((query_visibility)? (= syn::Visibility)!),
        ((delete_visibility)? (= syn::Visibility)!),
        ((insert_visibility)? (= syn::Visibility)!),
        ((select_visibility)? (= syn::Visibility)!),
        ((update_visibility)? (= syn::Visibility)!),

        ((filter)* (= String)+),
        ((delete_filter)* (= String)+),
        ((select_filter)* (= String)+),
        ((update_filter)* (= String)+),

        ((dynamic)?),
        ((optional)? (= Optionals)?),
        ((delete_optional)? (= Optionals)?),
        ((insert_optional)? (= Optionals)?),
        ((select_optional)? (= Optionals)?),
        ((update_optional)? (= Optionals)?),
        ((serde_double_option)?),

        ((returning)? (= safe::Returning)?),
        ((delete_returning)? (= safe::Returning)?),
        ((insert_returning)? (= safe::Returning)?),
        ((update_returning)? (= safe::Returning)?),

        ((unchecked)? (= Checks)?),
        ((krate as "crate")? (= syn::Path)!),
        ((print)? (= Print)?),
        ((debug)? (= Print)?),
    }
    pub QueryField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((select)* (= String)+),
        ((insert)* (= String)+),
        ((update)* (= String)+),

        ((filter)* (= String)+),
        ((delete_filter)* (= String)+),
        ((select_filter)* (= String)+),
        ((update_filter)* (= String)+),

        ((optional)? (= bool)?),
        ((delete_optional)? (= bool)?),
        ((insert_optional)? (= bool)?),
        ((select_optional)? (= bool)?),
        ((update_optional)? (= bool)?),

        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((foreign)* (= String)*),
        ((target)? (= safe::Named)!),

        ((named)? (= syn::Ident)!),
        ((typed)? (= syn::Type)!),

        ((from)? (= syn::Type)!),
        ((default)? (= syn::Expr)?),

        ((skip)? (= Skips)*),
        ((key)? (= Keys)*),
    }
}



impl QueryTable {

    pub fn init(mut self) -> Result<Self> {
        let a = &self.attr;
        for (r#type, attr, derive, visibility, filter, optional, returning) in [
            (Types::Delete, &a.delete, &a.delete_derive, &a.delete_visibility, &a.delete_filter, &a.delete_optional, &a.delete_returning),
            (Types::Insert, &a.insert, &a.insert_derive, &a.insert_visibility, &vec![]         , &a.insert_optional, &a.insert_returning),
            (Types::Select, &a.select, &a.select_derive, &a.select_visibility, &a.select_filter, &a.select_optional, &None              ),
            (Types::Update, &a.update, &a.update_derive, &a.update_visibility, &a.update_filter, &a.update_optional, &a.update_returning),
        ] {
            if attr.is_none() {
                if let Some(span) = spany!(derive, visibility, filter, optional, returning) {
                    let msg = format!("unused attribute: requires #[sqly({})]", r#type);
                    return Err(syn::Error::new(span, msg));
                }
            }
        }

        if self.attr.flat.is_none() {
            if let Some(span) = spany!(a.from_flat, a.flat_row) {
                let msg = "unused attribute: requires #[sqly(flat)]";
                return Err(syn::Error::new(span, msg));
            }
        }

        for field in &self.fields {
            self.list(&field.attr.key)?;
            self.list(&field.attr.skip)?;

            let listed = (&field.attr.key, &field.attr.skip);
            if let (Some(keys), Some(skips)) = listed {
                if skips.data.is_empty() {
                    let msg = match keys.data.len() {
                        0 => "conflicting attributes: #[sqly(skip, key)]",
                        _ => "ambiguous attributes: #[sqly(skip, key = ...)]",
                    };
                    return Err(syn::Error::new(skips.span, msg));
                }
                for skip in &skips.data {
                    if let Ok(r#type) = Types::try_from(skip.data) {
                        if keys.data.iter().any(|key| r#type == key.data.into()) {
                            let msg = format!(
                                "conflicting attributes: #[sqly(skip = {}, key = {})]",
                                r#type, r#type,
                            );
                            return Err(syn::Error::new(skip.span(), msg));
                        }
                    }
                }
            }
        }

        for field in &self.fields {
            let b = &field.attr;
            for (r#type, attr, value, filter, optional) in [
                (Types::Delete, &a.delete, &vec![],   &b.delete_filter, &b.delete_optional),
                (Types::Insert, &a.insert, &b.insert, &vec![]         , &b.insert_optional),
                (Types::Select, &a.select, &vec![],   &b.select_filter, &b.select_optional),
                (Types::Update, &a.update, &b.update, &b.update_filter, &b.update_optional),
            ] {
                if let Some(span) = spany!(value, filter, optional) {
                    if attr.is_none() {
                        let msg = format!("unused attribute: requires #[sqly({})] on struct", r#type);
                        return Err(syn::Error::new(span, msg));
                    }
                    if !self.fielded(field, r#type) {
                        let msg = format!("unused attribute: field not included in #[sqly({})]", r#type);
                        return Err(syn::Error::new(span, msg));
                    }
                }
            }
            if field.attr.key.is_none() {
                if let Some(span) = field.attr.filter.spany() {
                    let msg = "unused attribute: requires #[sqly(key)]";
                    return Err(syn::Error::new(span, msg));
                }
            }
        }

        for field in &self.fields {
            if self.foreign(field)?.is_some() {
                if let Some(skips) = &field.attr.skip {
                    if skips.data.is_empty() {
                        let msg = "conflicting attributes: #[sqly(skip, foreign)]";
                        return Err(syn::Error::new(skips.span, msg));
                    }
                    if let Some(skip) = skips.data.iter().find(|skip| skip.data == Skips::FromRow) {
                        let msg = "conflicting attributes: #[sqly(skip, foreign)]";
                        return Err(syn::Error::new(skip.span(), msg));
                    }
                }
                if let Some(span) = field.attr.select.spany() {
                    let msg = "conflicting attributes: #[sqly(foreign, select)]";
                    return Err(syn::Error::new(span, msg));
                }
            } else {
                if let Some(span) = field.attr.target.spany() {
                    let msg = "unused attribute: requires #[sqly(foreign)]";
                    return Err(syn::Error::new(span, msg));
                }
            }
        }

        #[cfg(not(feature = "serde"))]
        if let Some(span) = self.attr.serde_double_option.spany() {
            let msg = "unused attribute: requires the serde feature";
            return Err(syn::Error::new(span, msg));
        }

        let opt = [
            (Types::Delete, &a.delete),
            (Types::Insert, &a.insert),
            (Types::Select, &a.select),
            (Types::Update, &a.update),
        ].iter().filter(|(_, attr)| attr.is_some()).flat_map(|(r#type, _)| {
            self.fields.iter().filter(|f| self.fielded(f, *r#type)).map(|f| (f, *r#type))
        }).filter_map(|(field, r#type)| self.optional(field, r#type)).next();
        r#static(self.attr.dynamic.spany(), opt)?;

        let a = &mut self.attr;
        for returning in [
            &mut a.returning,
            &mut a.delete_returning,
            &mut a.insert_returning,
            &mut a.update_returning
        ] {
            if let Some(name) = returning {
                if let Some(info) = &mut name.data {
                    if let Some(table) = &mut info.data.table {
                        if table.is_ident("Self") {
                            let mut ident = self.ident.clone();
                            ident.set_span(table.spanned());
                            *table = ident.into();
                        }
                    }
                }
            }
        }

        Ok(self)
    }

    fn list<T>(&self, list: &Option<Name<Vec<Info<T>>>>) -> Result<()>
    where T: TryInto<Types> + ToString + PartialEq + Copy + Spanned {
        let types = self.types()?.collect::<Vec<_>>();

        if let Some(list) = list {
            for item in &list.data {
                if let Ok(r#type) = item.data.try_into() {
                    if !types.contains(&r#type) {
                        let name = item.data.to_string().to_lowercase();
                        let msg = format!("unused value: requires #[sqly({})] on struct", name);
                        return Err(syn::Error::new(item.span(), msg));
                    }
                }
            }
            for (i, item) in list.data.iter().enumerate() {
                let mut rest = list.data.as_slice()[i + 1..].iter();
                if let Some(item) = rest.find(|i| i.data == item.data) {
                    let msg = format!("duplicate value: {}", item.data.to_string());
                    return Err(syn::Error::new(item.span(), msg));
                }
            }
        }

        Ok(())
    }

}

pub fn r#static(dynamic: Option<Span>, optional: Option<Span>) -> Result<()> {
    match dynamic {
        Some(span) => if optional.is_none() {
            let msg = "unused attribute: queries do not need to be generated at runtime\
                \nnote: no fields end up parsed as #[sqly(optional)] in generated queries,\
                \n      remove #[sqly(dynamic)] to indicate static queries can be generated";
            return Err(syn::Error::new(span, msg));
        }
        None => if let Some(span) = optional {
            let msg = "unused attribute: requires #[sqly(dynamic)] on struct\
                \nnote: due to #[sqly(optional)] queries must be generated at runtime,\
                \n      use #[sqly(dynamic)] to explicitly opt-in for this behavior";
            return Err(syn::Error::new(span, msg));
        }
    }
    Ok(())
}



impl Rename {

    pub fn rename<'c>(&self, str: &'c str) -> Cow<'c, str> {
        use heck::*;
        Cow::Owned(match self {
            Rename::None => return Cow::Borrowed(str),
            Rename::LowerCase => str.to_lowercase(),
            Rename::UpperCase => str.to_uppercase(),
            Rename::CamelCase => str.to_lower_camel_case(),
            Rename::PascalCase => str.to_upper_camel_case(),
            Rename::SnakeCase => str.to_snake_case(),
            Rename::KebabCase => str.to_kebab_case(),
            Rename::UpperSnakeCase => str.to_shouty_snake_case(),
            Rename::UpperKebabCase => str.to_shouty_kebab_case(),
        })
    }

}

impl std::fmt::Display for Named {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Named::Ident(ident) => write!(f, "{}", ident),
            Named::String(string) => write!(f, "\"{}\"", string),
        }
    }

}

impl Name<Option<Info<Print>>> {

    pub fn output(&self, str: &str) -> proc_macro2::TokenStream {
        match self.data.as_ref().map(|data| data.data) {
            Some(Print::StdErr) => eprintln!("{}", str),
            Some(Print::StdOut) => println!("{}", str),
            Some(Print::Warn) => {
                let warn = format!("\n{}", str);
                let span = syn::Error::new_spanned(self, "").span();
                return quote::quote_spanned!(span =>
                    #[doc(hidden)]
                    #[deprecated = #warn]
                    macro_rules! sqly_print { () => {} }
                    sqly_print!();
                );
            }
            Some(Print::Panic) | None => {
                let err = syn::Error::new_spanned(self, str);
                return err.into_compile_error();
            }
        }
        proc_macro2::TokenStream::new()
    }

}



impl Default for Returning {
    fn default() -> Self {
        Self {
            table: Default::default(),
            fields: Default::default(),
        }
    }
}

impl Clone for Returning {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            fields: self.fields.clone(),
        }
    }
}

impl syn::parse::Parse for Returning {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let name = <syn::Ident as syn::ext::IdentExt>::peek_any;
        let table = match input.peek(name) {
            true => Some(input.parse()?),
            false => None,
        };
        let mut fields = Vec::<syn::Ident>::new();
        if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            fields.push(content.parse()?);
            while !content.is_empty() {
                content.parse::<syn::Token![,]>()?;
                if !content.is_empty() {
                    fields.push(content.parse()?);
                }
            }
        };
        if table.is_none() && fields.is_empty() {
            let look = input.lookahead1();
            look.peek(syn::Ident);
            look.peek(syn::token::Brace);
            return Err(look.error());
        }
        Ok(Self { table, fields })
    }
}

impl quote::ToTokens for Returning {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(table) = &self.table {
            table.to_tokens(tokens);
        }
        if let Some((first, rest)) = self.fields.split_first() {
            let brace = syn::token::Brace::default();
            brace.surround(tokens, |tokens| {
                first.to_tokens(tokens);
                let spacing = proc_macro2::Spacing::Alone;
                let char = proc_macro2::Punct::new(',', spacing);
                for field in rest {
                    char.to_tokens(tokens);
                    field.to_tokens(tokens);
                }
            });
        }
    }
}
