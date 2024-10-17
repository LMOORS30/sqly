use super::*;



parse! {
    pub UpdateTable {
        ((table)! (= syn::Path)!),
        ((rename)? (= Rename)!),

        ((unchecked)?),
        ((print)?),
        ((debug)?),
    }
    pub UpdateField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((skip)?),
        ((key)?),
    }
}



impl UpdateTable {

    pub fn init(self) -> Result<Self> {
        if let Some(field) = self.fields.iter().find(|field| {
            field.attr.skip.is_some() &&
            field.attr.key.is_some()
        }) {
            let span = field.attr.skip.as_ref().unwrap().span;
            let msg = "conflicting attributes: #[sqly(skip, key)]";
            return Err(syn::Error::new(span, msg));
        }

        if self.fields()?.all(|field| {
            field.attr.key.is_some()
        }) {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing update value";
            return Err(syn::Error::new(span, msg));
        }

        if self.fields()?.all(|field| {
            field.attr.key.is_none()
        }) {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing update key";
            return Err(syn::Error::new(span, msg));
        }

        Ok(self)
    }

}
