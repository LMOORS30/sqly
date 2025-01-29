use super::*;



parse! {
    pub UpdateTable {
        ((table)! (= syn::Path)!),
        ((rename)? (= Rename)!),

        ((filter)* (= String)+),

        ((unchecked)?),
        ((print)?),
        ((debug)?),
    }
    pub UpdateField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((update)* (= String)+),
        ((filter)* (= String)+),
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
                if !field.attr.filter.is_empty() {
                    let msg = "conflicting attributes: #[sqly(skip, filter)]";
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

        if self.fields()?.all(|field| {
            field.attr.key.is_some()
        }) {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing update value";
            return Err(syn::Error::new(span, msg));
        }

        if self.attr.filter.is_empty() && (
            self.fields()?.all(|field| {
                field.attr.key.is_none()
            })
        ) {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing update key";
            return Err(syn::Error::new(span, msg));
        }

        Ok(self)
    }

}
