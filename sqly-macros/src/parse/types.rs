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
        (Query = query),
        (Delete = delete),
        (Insert = insert),
        (Select = select),
        (Update = update),
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

parse! {
    pub QueryTable {
        ((table)! (= String)!),
        ((rename)? (= Rename)!),

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

        ((unchecked)?),
        ((print)?),
        ((debug)?),
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

        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((foreign)* (= String)*),
        ((target)? (= safe::Named)!),

        ((named)? (= syn::Ident)!),
        ((typed)? (= syn::Type)!),

        ((default)? (= syn::Path)?),
        ((from)? (= syn::Type)!),

        ((skip)? (= Skips)*),
        ((key)? (= Keys)*),
    }
}



impl QueryTable {

    pub fn init(self) -> Result<Self> {
        let a = &self.attr;
        for (r#type, attr, derive, visibility, filter) in [
            (Types::Delete, &a.delete, &a.delete_derive, &a.delete_visibility, &a.delete_filter),
            (Types::Insert, &a.insert, &a.insert_derive, &a.insert_visibility, &vec![],        ),
            (Types::Select, &a.select, &a.select_derive, &a.select_visibility, &a.select_filter),
            (Types::Update, &a.update, &a.update_derive, &a.update_visibility, &a.update_filter),
        ] {
            if attr.is_none() {
                if let Some(span) = spany!(derive, visibility, filter) {
                    let msg = format!("unused attribute: requires #[sqly({})]", r#type);
                    return Err(syn::Error::new(span, msg));
                }
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
                            let msg = "conflicting attributes: #[sqly(skip, key)]";
                            return Err(syn::Error::new(skip.span, msg));
                        }
                    }
                }
            }
        }

        for field in &self.fields {
            let b = &field.attr;
            for (r#type, attr, value, filter) in [
                (Types::Delete, &a.delete, &vec![],   &b.delete_filter),
                (Types::Insert, &a.insert, &b.insert, &vec![],        ),
                (Types::Select, &a.select, &vec![],   &b.select_filter),
                (Types::Update, &a.update, &b.update, &b.update_filter),
            ] {
                if let Some(span) = spany!(value, filter) {
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
            match self.foreign(field)? {
                Some(_) => {
                    if let Some(skips) = &field.attr.skip {
                        if skips.data.is_empty() {
                            let msg = "conflicting attributes: #[sqly(skip, foreign)]";
                            return Err(syn::Error::new(skips.span, msg));
                        }
                        if let Some(skip) = skips.data.iter().find(|skip| skip.data == Skips::Query) {
                            let msg = "conflicting attributes: #[sqly(skip, foreign)]";
                            return Err(syn::Error::new(skip.span, msg));
                        }
                    }
                    if let Some(select) = field.attr.select.first() {
                        let msg = "conflicting attributes: #[sqly(foreign, select)]";
                        return Err(syn::Error::new(select.span, msg));
                    }
                }
                None => {
                    if let Some(span) = field.attr.target.spany() {
                        let msg = "unused attribute: requires #[sqly(foreign)]";
                        return Err(syn::Error::new(span, msg));
                    }
                }
            }
        }

        Ok(self)
    }

    fn list<T>(&self, list: &Option<Name<Vec<Info<T>>>>) -> Result<()>
    where T: TryInto<Types> + ToString + PartialEq + Copy {
        let types = self.types()?.collect::<Vec<_>>();

        if let Some(list) = list {
            for item in &list.data {
                if let Ok(r#type) = item.data.try_into() {
                    if !types.contains(&r#type) {
                        let name = item.data.to_string();
                        let msg = format!("unused value: requires #[sqly({})] on struct", name);
                        return Err(syn::Error::new(item.span, msg));
                    }
                }
            }
            for (i, item) in list.data.iter().enumerate() {
                let mut rest = list.data.as_slice()[i + 1..].iter();
                if let Some(item) = rest.find(|i| i.data == item.data) {
                    let name = item.data.to_string();
                    let msg = format!("duplicate value: {}", name);
                    return Err(syn::Error::new(item.span, msg));
                }
            }
        }

        Ok(())
    }

}



impl Rename {

    pub fn rename(&self, str: &str) -> String {
        use heck::*;
        match self {
            Rename::None => str.to_string(),
            Rename::LowerCase => str.to_lowercase(),
            Rename::UpperCase => str.to_uppercase(),
            Rename::CamelCase => str.to_lower_camel_case(),
            Rename::PascalCase => str.to_upper_camel_case(),
            Rename::SnakeCase => str.to_snake_case(),
            Rename::KebabCase => str.to_kebab_case(),
            Rename::UpperSnakeCase => str.to_shouty_snake_case(),
            Rename::UpperKebabCase => str.to_shouty_kebab_case(),
        }
    }

}



impl std::fmt::Display for Named {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Named::Ident(ident) => write!(f, "{}", ident),
            Named::String(string) => write!(f, "\"{}\"", string),
        }
    }

}
