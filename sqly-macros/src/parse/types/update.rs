use super::*;



parse! {
    pub UpdateTable {
        ((table)! (= safe::Paved)!),
        ((rename_all)? (= Rename)!),

        ((dynamic)?),
        ((optional)?),
        ((keyless)?),
        ((filter)* (= String)+),
        ((returning)? (= safe::Returning)?),

        ((unchecked)? (= Checks)?),
        ((krate as "crate")? (= syn::Path)!),
        ((print)? (= Print)?),
        ((debug)? (= Print)?),
    }
    pub UpdateField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((update)* (= String)+),
        ((filter)* (= String)+),
        ((optional)? (= bool)?),
        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((skip)?),
        ((key)?),
    }
}



impl UpdateTable {

    pub fn init(self) -> Result<Self> {
        for field in &self.fields {
            if let Some(skip) = &field.attr.skip {
                if field.attr.key.is_some() {
                    let msg = "conflicting attributes: #[sqly(skip, key)]";
                    return Err(syn::Error::new(skip.span, msg));
                }
                if !field.attr.update.is_empty() {
                    let msg = "conflicting attributes: #[sqly(skip, update)]";
                    return Err(syn::Error::new(skip.span, msg));
                }
            }
            if let Some(key) = &field.attr.key {
                if !field.attr.update.is_empty() {
                    let msg = "conflicting attributes: #[sqly(key, update)]";
                    return Err(syn::Error::new(key.span, msg));
                }
            } else {
                if let Some(span) = field.attr.filter.spany() {
                    let msg = "unused attribute: requires #[sqly(key)]";
                    return Err(syn::Error::new(span, msg));
                }
            }
        }

        if self.fields().all(|field| field.attr.key.is_some()) {
            let msg = "incomplete query: missing update value";
            return Err(syn::Error::new(Span::call_site(), msg));
        }

        if let Some(keyless) = &self.attr.keyless {
            if self.fields().any(|field| field.attr.key.is_some()) {
                let msg = "conflicting attributes: #[sqly(keyless)] with #[sqly(key)]\n\
                    help: remove #[sqly(keyless)]";
                return Err(syn::Error::new(keyless.span, msg));
            }
        } else {
            if self.fields().all(|field| field.attr.key.is_none()) {
                let msg = "incomplete query: missing update key\n\
                    help: use #[sqly(keyless)] if intended";
                return Err(syn::Error::new(Span::call_site(), msg));
            }
        }

        self.r#static()?;
        Ok(self)
    }

}
