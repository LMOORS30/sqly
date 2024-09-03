use syn::Result;

mod delete;
mod insert;
mod select;
mod update;

pub use delete::*;
pub use insert::*;
pub use select::*;
pub use update::*;

use super::rules::*;



vars! {
    pub Types {
        (Delete = delete),
        (Insert = insert),
        (Select = select),
        (Update = update),
    }
}

vars! {
    pub Skips: Types {
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
        (CamelCase = "lowerCamelCase"),
        (PascalCase = "UpperCamelCase"),
        (SnakeCase = "snake_case"),
        (KebabCase = "kebab-case"),
        (UpperSnakeCase = "SCREAMING_SNAKE_CASE"),
        (UpperKebabCase = "SCREAMING-KEBAB-CASE"),
    }
}

parse! {
    pub QueryTable {
        ((table)! (= String)!),
        ((rename)? (= Rename)!),

        ((delete)? (= syn::Ident)?),
        ((insert)? (= syn::Ident)?),
        ((select)? (= syn::Ident)?),
        ((update)? (= syn::Ident)?),

        ((query_derive)* (= syn::Path)+),
        ((delete_derive)* (= syn::Path)+),
        ((insert_derive)* (= syn::Path)+),
        ((select_derive)* (= syn::Path)+),
        ((update_derive)* (= syn::Path)+),

        ((query_visibility)? (= syn::Visibility)!),
        ((delete_visibility)? (= syn::Visibility)!),
        ((insert_visibility)? (= syn::Visibility)!),
        ((select_visibility)? (= syn::Visibility)!),
        ((update_visibility)? (= syn::Visibility)!),

        ((print)?),
        ((debug)?),
    }
    pub QueryField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((skip)? (= Skips)*),
        ((key)? (= Keys)*),
    }
}



impl QueryTable {

    pub fn init(self) -> Result<Self> {
        for (name, attr, derive, visibility) in [
            ("delete", &self.attr.delete, &self.attr.delete_derive, &self.attr.delete_visibility),
            ("insert", &self.attr.insert, &self.attr.insert_derive, &self.attr.insert_visibility),
            ("select", &self.attr.select, &self.attr.select_derive, &self.attr.select_visibility),
            ("update", &self.attr.update, &self.attr.update_derive, &self.attr.update_visibility),
        ] {
            if attr.is_none() {
                let visibility = visibility.as_ref().map(|visibility| visibility.span);
                let derive = derive.first().map(|derive| derive.span);
                if let Some(span) = derive.or(visibility) {
                    let msg = format!("unused attribute: requires #[sqly({})]", name);
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
                    let span = skips.span;
                    let msg = match keys.data.len() {
                        0 => "conflicting attributes: #[sqly(skip, key)]",
                        _ => "ambiguous attributes: #[sqly(skip, key = ...)]",
                    };
                    return Err(syn::Error::new(span, msg));
                }
                for skip in &skips.data {
                    let typed = Types::from(skip.data);
                    if keys.data.iter().any(|key| typed == key.data.into()) {
                        let span = skip.span;
                        let msg = "conflicting attributes: #[sqly(skip, key)]";
                        return Err(syn::Error::new(span, msg));
                    }
                }
            }
        }

        Ok(self)
    }

    fn list<T>(&self, list: &Option<Name<Vec<Info<T>>>>) -> Result<()>
    where T: TryFrom<Types> + ToString + PartialEq + Copy {
        let types = self.types()?.flat_map(|types| {
            T::try_from(types).ok()
        }).collect::<Vec<_>>();

        if let Some(list) = list {
            for item in &list.data {
                if !types.contains(&item.data) {
                    let span = item.span;
                    let name = item.data.to_string();
                    let msg = format!("unused value: requires #[sqly({})] on struct", name);
                    return Err(syn::Error::new(span, msg));
                }
            }
            for (i, item) in list.data.iter().enumerate() {
                let mut rest = list.data.as_slice()[i + 1..].iter();
                if let Some(item) = rest.find(|i| i.data == item.data) {
                    let span = item.span;
                    let name = item.data.to_string();
                    let msg = format!("duplicate value: {}", name);
                    return Err(syn::Error::new(span, msg));
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
