use super::*;



parse! {
    pub InsertTable {
        ((table)! (= syn::Path)!),
        ((rename)? (= Rename)!),

        ((unchecked)?),
        ((print)?),
        ((debug)?),
    }
    pub InsertField {
        ((column)? (= String)!),
        ((rename)? (= Rename)!),

        ((insert)* (= String)+),
        ((value)? (= syn::Expr)!),
        ((infer)?),

        ((skip)?),
    }
}



impl InsertTable {

    pub fn init(self) -> Result<Self> {
        for field in &self.fields {
            if let Some(skip) = &field.attr.skip {
                if !field.attr.insert.is_empty() {
                    let msg = "conflicting attributes: #[sqly(skip, insert)]";
                    return Err(syn::Error::new(skip.span, msg));
                }
            }
        }

        if self.fields()?.next().is_none() {
            let span = proc_macro2::Span::call_site();
            let msg = "incomplete query: missing insert value";
            return Err(syn::Error::new(span, msg));
        }

        Ok(self)
    }

}
